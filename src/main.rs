mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod slug;
mod state;

use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tokio::net::TcpListener;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

#[tokio::main()]
async fn main() {
    dotenvy::dotenv().ok();

    let config = config::Config::from_env();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let session_store = PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("Failed to migrate session store");
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let state = AppState { db: pool, config };
    let app = routes::app(state, session_layer);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
