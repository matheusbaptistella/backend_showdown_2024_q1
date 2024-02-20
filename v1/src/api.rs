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
/// * 
pub async fn add_transaction(
    Extension(cnn): Extension<PgPool>,
    Path(id): Path<i32>,
    Json(txn): Json<Transaction>,
) -> Result<StatusCode, StatusCode> {
    match crate::db::add_transaction(&cnn, &txn, id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            match e {
                // Client id not found
                sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
                // Balance limit exceeded
                sqlx::Error::Database(db_err) => match db_err.constraint() {
                    Some(constraint) if constraint == "balance_exceeded" => {
                        Err(StatusCode::UNPROCESSABLE_ENTITY)
                    }
                    _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
                },
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
