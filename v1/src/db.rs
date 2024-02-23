use anyhow::Result;
//use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

/// Represents all the fields of  a transaction.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, FromRow, Clone)]
pub struct Transaction {
    pub client_id: i32,
    pub txn_value: i32,
    pub txn_type: String,        // Limit 1
    pub txn_description: String, // Limit 10
    pub executed_at: String,
}

// Usar uma na outra depois dos testes

/// Represents only the information that should be provided via json to write a
/// Transaction on the database.
#[derive(Debug, Serialize, Deserialize, Clone)] //  remover debug e clone
pub struct CoreTransaction {
    pub txn_value: i32,
    pub txn_type: String,        // Limit 1
    pub txn_description: String, // Limit 10
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct RespTransaction {
    pub limit: i32,
    pub balance: i32,
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
/// * `core_txn` - a transaction with only the necessary fields.
/// * `id` - the id of the client to reference the transaction.
///
/// ## Returns
/// * A tuple containing the total_limit of the client and their updated balance.
pub async fn add_transaction(
    pool: &PgPool,
    core_txn: &CoreTransaction,
    id: i32,
) -> Result<RespTransaction, sqlx::Error> {
    let transaction = pool.begin().await?;

    // If the txn_value would exceed the balance limit (a constraint in the db), the transaction is cancelled before
    // insertion happens. Otherwise, updates the clients balance.
    let resp = sqlx::query_as::<_, RespTransaction>(
        "UPDATE clients SET balance = balance - $1 WHERE client_id = $2 RETURNING total_limit, balance"
    )
    .bind(core_txn.txn_value)
    .bind(id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO transaction (client_id, txn_value, txn_type, txn_description)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(core_txn.txn_value)
    .bind(core_txn.txn_type.to_string())
    .bind(core_txn.txn_description.to_string())
    //.bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    // Only commit the changes if all queries were successful
    transaction.commit().await?;

    Ok(resp)
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
        let core_txn = CoreTransaction {
            txn_value: 50000,
            txn_type: "d".to_string(),
            txn_description: "test".to_string(),
        };
        let r = add_transaction(&connection_pool, &core_txn, 1).await;

        assert!(
            matches!(
                r,
                Ok(RespTransaction {
                    limit: 100000,
                    balance: 50000,
                })
            ),
            "Expected a RespTransaction instance with an updated balance, but got {:?}",
            r
        );
    }

    #[sqlx::test]
    async fn add_transaction_invalid_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let core_txn = CoreTransaction {
            txn_value: 50000,
            txn_type: "d".to_string(),
            txn_description: "test".to_string(),
        };
        let r = add_transaction(&connection_pool, &core_txn, 6).await;

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
        let core_txn = CoreTransaction {
            txn_value: 200001,
            txn_type: "d".to_string(),
            txn_description: "test".to_string(),
        };
        let r = add_transaction(&connection_pool, &core_txn, 1).await;

        assert!(
            matches!(r, Err(sqlx::Error::Database(_))),
            "Expected Database error, but got {:?}",
            r
        );
    }

    #[sqlx::test]
    async fn get_bank_statement_valid_id() {
        dotenv::dotenv().ok();
        let connection_pool = init_db().await.unwrap();
        let core_txn = CoreTransaction {
            txn_value: 50000,
            txn_type: "d".to_string(),
            txn_description: "test".to_string(),
        };
        let _ = add_transaction(&connection_pool, &core_txn, 1)
            .await
            .unwrap();
        let r = get_bank_statement(&connection_pool, 1).await.unwrap();
        let t = vec![Transaction {
            client_id: 1,
            txn_value: 50000,
            txn_type: "d".to_string(),
            txn_description: "test".to_string(),
            executed_at: r[0].executed_at.clone(),
        }];

        assert_eq!(r, t, "Expected bank statement, but got {:?}", r);
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
