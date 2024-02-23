use crate::db::{CoreTransaction, RespTransaction, Transaction};
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    Extension,
};
use serde_json::{json, Value};
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
) -> Result<Json<RespTransaction>, StatusCode> {
    match crate::db::add_transaction(&cnn, &core_txn, id).await {
        Ok(resp_transaction) => Ok(Json(resp_transaction)),
        Err(e) => {
            match e {
                // Client id not found (404)
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                // Any database error related to violating the limit_exceeded contraint, a transaction value that is not
                // an integer, or a transaction type/description that is not supported are all treated as
                // UNPROCESSABLE_ENTITY (422)
                sqlx::Error::Database(_db_err) => Err(StatusCode::UNPROCESSABLE_ENTITY),
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
/// * On success, a Json containing the latest 10 transactions, or a status code
/// related to the error that happened.
pub async fn get_bank_statement(
    Extension(cnn): Extension<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Transaction>>, StatusCode> {
    match crate::db::get_bank_statement(&cnn, id).await {
        Ok(transactions) => Ok(Json(transactions)),
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

    async fn setup_test() -> TestClient {
        
    }

    async fn add_transaction_valid_id() {
        dotenv::dotenv().ok();
        let connection_pool = crate::db::init_db().await.unwrap();
        let core_txn = CoreTransaction {
            txn_value: 50000,
            txn_type: "d".to_string(),
            txn_description: "test".to_string(),
        };
    }

    async fn get_bank_statement() {
        todo!()
    }
}
