use crate::model::{AccountSummary, AccountSummaryInfo, ClientInfo, CoreTransaction, Transaction};
use sqlx::PgPool;

/// Create a database connection pool.
///
/// ## Returns
/// * A connection pool.
pub async fn init_db() -> anyhow::Result<PgPool> {
    // Create the connection pool
    let database_url = std::env::var("DATABASE_URL")?;
    let connection_pool = PgPool::connect(&database_url).await?;

    // Initialize the database for local tests
    #[cfg(debug_assertions)]
    sqlx::migrate!("./migrations").run(&connection_pool).await?;

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
/// * The client's limit and updated balance information.
pub async fn add_transaction(
    pool: &PgPool,
    core_txn: &CoreTransaction,
    id: i32,
) -> Result<ClientInfo, sqlx::Error> {
    let transaction = pool.begin().await?;

    // If the txn_value would exceed the balance limit (a constraint in the db), the transaction is cancelled before
    // insertion happens. Otherwise, updates the clients balance.
    let resp: ClientInfo;
    if core_txn.tipo == "c" {
        resp = sqlx::query_as::<_, ClientInfo>(
            "UPDATE clientes SET saldo = saldo + $1
            WHERE id = $2
            RETURNING limite, saldo",
        )
        .bind(core_txn.valor)
        .bind(id)
        .fetch_one(pool)
        .await?;
    } else {
        resp = sqlx::query_as::<_, ClientInfo>(
            "UPDATE clientes SET saldo = saldo - $1
            WHERE id = $2
            RETURNING limite, saldo",
        )
        .bind(core_txn.valor)
        .bind(id)
        .fetch_one(pool)
        .await?;
    }

    sqlx::query(
        "INSERT INTO transacoes(id, valor, tipo, descricao)
        VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(core_txn.valor)
    .bind(core_txn.tipo.to_string())
    .bind(core_txn.descricao.to_string())
    .execute(pool)
    .await?;

    // Only commit the changes if all queries were successful
    transaction.commit().await?;

    Ok(resp)
}

/// Get the account summary of a client (the 10 latest transactions).
///
/// ## Arguments
/// * `pool` - the database connection to use.
/// * `id` - the id of the client to reference the transaction.
///
/// ## Returns
/// * The client's account summary .
pub async fn get_account_summary(pool: &PgPool, id: i32) -> Result<AccountSummary, sqlx::Error> {
    // Fazer join pra ficar mais eficiente (talvez armazenar ao contrario?) tentar tudo numa query só?
    let bsinfo = sqlx::query_as::<_, AccountSummaryInfo>(
        "SELECT saldo AS total, CURRENT_TIMESTAMP AS data_extrato, limite
        FROM clientes
        WHERE id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT valor, tipo, descricao, realizada_em
        FROM transacoes
        WHERE id = $1
        ORDER BY realizada_em DESC
        LIMIT 10",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let account_s = AccountSummary {
        saldo: bsinfo,
        ultimas_transacoes: transactions,
    };

    Ok(account_s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_valid_id(pool: PgPool) {
        let core_txn = CoreTransaction {
            valor: 1000000,
            tipo: "d".to_string(),
            descricao: "test".to_string(),
        };
        let r = add_transaction(&pool, &core_txn, 3).await.unwrap();

        assert_eq!(
            r,
            ClientInfo {
                limite: 1000000,
                saldo: -1000000,
            }
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_invalid_id(pool: PgPool) {
        let core_txn = CoreTransaction {
            valor: 50000,
            tipo: "d".to_string(),
            descricao: "test".to_string(),
        };
        let r = add_transaction(&pool, &core_txn, 6).await;

        assert!(
            matches!(r, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound error, but got {:?}",
            r
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn add_transaction_balance_exceeded(pool: PgPool) {
        let core_txn = CoreTransaction {
            valor: 1000001,
            tipo: "d".to_string(),
            descricao: "test".to_string(),
        };
        let r = add_transaction(&pool, &core_txn, 3).await;

        assert!(
            matches!(r, Err(sqlx::Error::Database(_))),
            "Expected Database error, but got {:?}",
            r
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_account_summary_empty_valid_id(pool: PgPool) {
        let account_s = get_account_summary(&pool, 4).await.unwrap();

        assert_eq!(
            account_s,
            AccountSummary {
                saldo: AccountSummaryInfo {
                    total: 0,
                    data_extrato: account_s.saldo.data_extrato.clone(),
                    limite: 10000000,
                },
                ultimas_transacoes: Vec::new()
            }
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_account_summary_single_valid_id(pool: PgPool) {
        let core_txn = CoreTransaction {
            valor: 100,
            tipo: "d".to_string(),
            descricao: "test".to_string(),
        };
        let _ = add_transaction(&pool, &core_txn, 4).await.unwrap();
        let account_s = get_account_summary(&pool, 4).await.unwrap();

        assert_eq!(
            account_s,
            AccountSummary {
                saldo: AccountSummaryInfo {
                    total: -100,
                    data_extrato: account_s.saldo.data_extrato.clone(),
                    limite: 10000000,
                },
                ultimas_transacoes: vec![Transaction {
                    valor: core_txn.valor,
                    tipo: core_txn.tipo.clone(),
                    descricao: core_txn.descricao.clone(),
                    realizada_em: account_s.ultimas_transacoes[0].realizada_em.clone()
                }]
            }
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn get_account_summary_invalid_id(pool: PgPool) {
        let account_s = get_account_summary(&pool, 6).await;
        assert!(
            matches!(account_s, Err(sqlx::Error::RowNotFound)),
            "Expected RowNotFound error, but got {:?}",
            account_s
        );
    }
}
