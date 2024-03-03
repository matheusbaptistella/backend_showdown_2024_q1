use crate::model::{AccountSummary, ClientInfo, CoreTransaction};
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    Extension,
};
use sqlx::PgPool;

/// Add a transaction to the database and update the client's limit/balance.
///
/// ## Arguments
/// * `Extension(cnn)` - dependency injected by Axum from the database layer.
/// * `Path(id)` - the client id, parsed by Axum from the url path.
/// * `Json(core_txn)` - the transaction, parsed by Axum from the json body.
///
/// ## Returns
/// * On success, an OK status code followed by a Json body containing the
/// client's limit and updated balance. In the case of an error, a status code
/// related to it.
pub async fn add_transaction(
    Extension(cnn): Extension<PgPool>,
    Path(id): Path<i32>,
    Json(core_txn): Json<CoreTransaction>,
) -> Result<Json<ClientInfo>, StatusCode> {
    if core_txn.valor < 0
        || (core_txn.tipo != "c" && core_txn.tipo != "d")
        || (core_txn.descricao.len() < 1 || core_txn.descricao.len() > 10)
    {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    match crate::db::add_transaction(&cnn, &core_txn, id).await {
        Ok(resp_transaction) => Ok(Json(resp_transaction)),
        Err(e) => {
            match e {
                // Client id not found
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                // Any error related to not providing an input according to the database specifications
                sqlx::Error::Database(_db_err) => Err(StatusCode::UNPROCESSABLE_ENTITY),
                // Any other error
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Get the bank statement of a client (the 10 latest transactions).
///
/// ## Arguments
/// * `Extension(cnn)` - dependency injected by Axum from the database layer.
/// * `Path(id)` - the client id, parsed by Axum from the url path.
///
/// ## Returns
/// * On success, an OK status code followed by a Json body containing details
/// about the client's balance and its latest 10 transactions. In the case of an
/// error, a status code related to it.
pub async fn get_account_summary(
    Extension(cnn): Extension<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<AccountSummary>, StatusCode> {
    match crate::db::get_account_summary(&cnn, id).await {
        Ok(account_s) => Ok(Json(account_s)),
        Err(e) => {
            match e {
                // Client id not found
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                // Any other error
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_valid_id(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let r = server
            .post(&"/clientes/1/transacoes")
            .json(&json!({
                "valor": 50000,
                "tipo": "d",
                "descricao": "test"
            }))
            .await;

        assert_eq!(r.status_code(), StatusCode::OK);
        r.assert_json(&json!({
            "limite": 100000,
            "saldo": -50000
        }));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_invalid_id(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let r = server
            .post(&"/clientes/6/transacoes")
            .json(&json!({
                "valor": 50000,
                "tipo": "d",
                "descricao": "test"
            }))
            .await;

        assert_eq!(r.status_code(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_balance_exceeded(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let r = server
            .post(&"/clientes/1/transacoes")
            .json(&json!({
                "valor": 100001,
                "tipo": "d",
                "descricao": "test"
            }))
            .await;

        assert_eq!(r.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_invalid_json_field(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let r = server
            .post(&"/clientes/1/transacoes")
            .json(&json!({
                "valor": -100001,
                "tipo": "e",
                "descricao": ""
            }))
            .await;

        assert_eq!(r.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_account_summary_empty_valid_id(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let r = server.get(&"/clientes/2/extrato").await;
        let data_extrato = r.json::<serde_json::Value>()["saldo"]["data_extrato"].clone();

        r.assert_json(&json!({
            "saldo": {
                "total": 0,
                "data_extrato": data_extrato,
                "limite": 80000
              },
              "ultimas_transacoes": []
        }));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_account_summary_single_valid_id(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let _ = server
            .post(&"/clientes/2/transacoes")
            .json(&json!({
                "valor": 100,
                "tipo": "d",
                "descricao": "test"
            }))
            .await;
        let account_s = server.get(&"/clientes/2/extrato").await;
        let data_extrato = account_s.json::<serde_json::Value>()["saldo"]["data_extrato"].clone();
        let data_transacao =
            account_s.json::<serde_json::Value>()["ultimas_transacoes"][0]["realizada_em"].clone();

        account_s.assert_json(&json!({
            "saldo": {
                "total": -100,
                "data_extrato": data_extrato,
                "limite": 80000
              },
              "ultimas_transacoes": [
                {
                    "valor": 100,
                    "tipo": "d",
                    "descricao": "test",
                    "realizada_em": data_transacao
                }
              ]
        }));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_account_summary_invalid_id(pool: PgPool) {
        let app = crate::router(pool);
        let server = TestServer::new(app).unwrap();
        let r = server.get(&"/clientes/6/extrato").await;

        assert_eq!(r.status_code(), StatusCode::NOT_FOUND);
    }
}
