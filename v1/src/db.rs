use anyhow::Result;
//use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

/// Represents a transaction.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Transaction {
    pub client_id: i32,
    pub txn_value: i32,
    pub txn_type: String,        // Limit 1
    pub txn_description: String, // Limit 10
    pub executed_at: String,
}

/// Create a database connection pool. Run any migration.
///
/// ## Returns
/// * A connection pool.
pub async fn init_db() -> Result<PgPool> {
    // Create the connection pool
    let database_url = std::env::var("DATABASE_URL")?;
    let connection_pool = PgPool::connect(&database_url).await?;
    // Initialise the database by migrating the initial sql script
    sqlx::migrate!().run(&connection_pool).await?;

    Ok(connection_pool)
}

/// Add a transaction to the database and update the client's limit/balance.
///
/// ## Arguments
/// * `pool` - the database connection to use.
/// * `txn_value` - the value of the transaction.
/// * `txn_type` - the type of the transaction.
/// * `txn_description` - the deescription of the transaction.
/// * `id` - the id of the client to reference the transaction.
pub async fn add_transaction(
    pool: &PgPool,
    txn_value: i32,
    txn_type: String,
    txn_description: String,
    id: i32,
) -> Result<(i32, i32), sqlx::Error> {
    let transaction = pool.begin().await?;

    // If the txn_value would exceed the balance limit (a constraint in the db), the transaction is cancelled before
    // insertion happens. Otherwise, updates the clients balance.
    let row = sqlx::query("UPDATE clients SET balance = balance - $1 WHERE client_id = $2 RETURNING total_limit, balance")
        .bind(txn_value)
        .bind(id)
        .fetch_one(pool)
        .await?;

    sqlx::query(
        "INSERT INTO transaction (client_id, txn_value, txn_type, txn_description)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(txn_value)
    .bind(txn_type.to_string())
    .bind(txn_description.to_string())
    //.bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    // Only commit the changes if all queries were successful
    transaction.commit().await?;

    let total_limit: i32 = row.try_get("total_limit")?;
    let balance: i32 = row.try_get("balance")?;

    Ok((total_limit, balance))
}

/// Get the bank statement of a client (the 10 latest transactions).
///
/// ## Arguments
/// * `pool` - the database connection to use.
/// * `id` - the id of the client to reference the transaction.
///
/// ## Returns
/// * A vector containing the latest 10 transactions, or an error.
pub async fn get_bank_statement(pool: &PgPool, id: i32) -> Result<Vec<Transaction>, sqlx::Error> {
    // Fazer join pra ficar mais eficiente (talvez armazenar ao contrario?)
    let result = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE client_id = $1 ORDER BY executed_at DESC LIMIT 10",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test]
    async fn add_transaction_valid_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let r = add_transaction(
            &connection_pool,
            50000,
            "d".to_string(),
            "test".to_string(),
            1,
        )
        .await;
        assert!(matches!(r, Ok((100000, 50000))), "Expected (), but got {:?}", r);
    }

    #[sqlx::test]
    async fn add_transaction_invalid_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let r = add_transaction(
            &connection_pool,
            1000,
            "d".to_string(),
            "test".to_string(),
            6,
        )
        .await;
        assert!(
            matches!(r, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound error, but got {:?}",
            r
        );
    }

    #[sqlx::test]
    async fn add_transaction_balance_exceeded() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let r = add_transaction(
            &connection_pool,
            200001,
            "d".to_string(),
            "test".to_string(),
            1,
        )
        .await;
        assert!(
            matches!(r, Err(sqlx::Error::Database(db_err))),
            "Expected Database error, but got {:?}",
            r
        );
    }

    #[sqlx::test]
    async fn get_empty_bank_statement_valid_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let r = get_bank_statement(&connection_pool, 1).await;
        assert!(
            matches!(r, Ok(value) if value.is_empty()),
            "Expected empty result, but got {:?}",
            r
        );
    }

    #[sqlx::test]
    async fn get_bank_statement_invalid_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let r = get_bank_statement(&connection_pool, 6).await;
        assert!(
            matches!(r, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound error, but got {:?}",
            r
        );
    }
}
