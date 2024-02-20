use anyhow::Result;
use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;

mod api;
mod db;

// Build the web service router.
pub fn router(pool: PgPool) -> Router {
    Router::new()
        .nest_service("/cliente/:id/transacoes", post(api::add_transaction))
        .nest_service("/cliente/:id/extrato", get(api::get_bank_statement))
        // Add the connection pool as a "layer", available for dependency injection.
        .layer(Extension(pool))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env
    dotenv::dotenv().ok();

    // Initialise the Postgres database
    let connection_pool = db::init_db().await?;

    // Initialise Axum routing service
    let app = router(connection_pool);

    // Address to listen on
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?; // Talvez trocar para socket addr (?)

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
