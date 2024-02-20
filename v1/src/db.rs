use anyhow::Result;
//use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

/// Represents a transaction.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Transaction {
    pub clinet_id: i32,
    pub txn_value: i32,
    pub txn_type: String,        // Limite 1
    pub txn_description: String, // Limite 10
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
/// * `txn` - the transaction containing the data to be written.
/// * `id` - the id of the client to reference the transaction.
pub async fn add_transaction(pool: &PgPool, txn: &Transaction, id: i32) -> Result<(), sqlx::Error> {
    let transaction = pool.begin().await?;

    // If the txn_value would exceed the balance limit (a constraint in the db),
    // the transaction is cancelled before insertion happens. Otherwise, updates
    // the clients balance.
    sqlx::query("UPDATE clients SET balance = balance - $1 WHERE client id = $2")
        .bind(txn.txn_value)
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO transaction (client_id, txn_value, txn_type, txn_description, executed_at)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(txn.txn_value)
    .bind(txn.txn_type.to_string())
    .bind(txn.txn_description.to_string())
    //.bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    // Only commits the changes if all queries were successful
    transaction.commit().await?;

    Ok(())
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
    // Fazer join pra ficar mais eficiente e ta errada pq tem que ser as 10 ultimas
    Ok(
        sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE client_id = $1 LIMIT 10",
        )
        .bind(id)
        .fetch_all(pool)
        .await?,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    async fn add_transaction_valid_id() {
        todo!()
    }

    async fn add_transaction_invalid_id() {
        todo!()
    }

    async fn add_transaction_balance_exceeded() {
        todo!()
    }

    async fn get_balance_valid_id() {
        todo!()
    }

    async fn get_balance_invalid_id() {
        todo!()
    }
}