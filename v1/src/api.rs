use crate::db::Transaction;
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
/// * `Json(txn)` - the transaction, parsed by Axum from the json body.
///
/// ## Returns
/// * A status code depending on whether the insertion was successful or not,
/// and, in the case of an error, a status code related to it.
pub async fn add_transaction(
    Extension(cnn): Extension<PgPool>,
    Path(id): Path<i32>,
    Json(txn): Json<Transaction>,
) -> Result<StatusCode, StatusCode> {
    match crate::db::add_transaction(&cnn, &txn, id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            match e {
                // Client id not found (404)
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                // Any database error related to violating the limit_exceeded contraint, a transaction value that is not
                // an integer, or a transaction type/description that is not supported are all treated as
                // UNPROCESSABLE_ENTITY (422)
                sqlx::Error::Database(db_err) => Err(StatusCode::UNPROCESSABLE_ENTITY),
                // sqlx::Error::Database(db_err) => match db_err.constraint() {
                //     Some(constraint) if constraint == "limit_exceeded" => {
                //         Err(StatusCode::UNPROCESSABLE_ENTITY)
                //     }
                //     _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
                // },
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
/// * A Json containing the latest 10 transactions, or a status code related to
/// the error.
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
