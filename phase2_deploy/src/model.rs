use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type DbPool = Arc<Mutex<PgPool>>;

/// Represents a transaction.
#[derive(Debug, Serialize, PartialEq, FromRow)]
pub struct Transaction {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
    pub realizada_em: DateTime<Utc>,
}

/// Represents only the information that should be provided via json to write a
/// Transaction on the database.
#[derive(Serialize, Deserialize)]
pub struct CoreTransaction {
    pub valor: i32,
    pub tipo: String,
    pub descricao: String,
}

/// Information associated to a client.
#[derive(Debug, Serialize, PartialEq, FromRow)]
pub struct ClientInfo {
    pub limite: i32,
    pub saldo: i32,
}

/// Information related to a client's account.
#[derive(Debug, Serialize, PartialEq, FromRow)]
pub struct AccountSummaryInfo {
    pub total: i32,
    pub data_extrato: DateTime<Utc>,
    pub limite: i32,
}

/// Information about the client's balance and latest transactions.
#[derive(Debug, Serialize, PartialEq)]
pub struct AccountSummary {
    pub saldo: AccountSummaryInfo,
    pub ultimas_transacoes: Vec<Transaction>,
}
