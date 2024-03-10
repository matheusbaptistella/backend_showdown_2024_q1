use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

mod api;
mod db;
mod model;

// Build the web service router.
pub fn router(app_state: model::DbPool) -> Router {
    Router::new()
        .route("/clientes/:id/transacoes", post(api::add_transaction))
        .route("/clientes/:id/extrato", get(api::get_account_summary))
        .with_state(app_state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the Postgres database connection
    let connection_pool = db::init_db().await?;
    let app_state = Arc::new(Mutex::new(connection_pool));
    let app = router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
