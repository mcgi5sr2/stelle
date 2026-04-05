mod config;
mod error;
mod handlers;
mod models;
mod slug;
mod state;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;
use state::AppState;

#[tokio::main()]
async fn main() {
    dotenvy::dotenv().ok();

    let config = config::Config::from_env();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.expect("Failed to migrate session store");
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false);

    let app = Router::new()
        // slug route this is where user are sent
        .route("/e/{slug}", get(handlers::show_exhibit))
        .route("/admin/pages/new", get(handlers::new_page_form))
        .route("/admin/pages", post(handlers::create_page))
        .route("/admin/pages/{slug}/qr", get(handlers::generate_qr))
        .route("/login", get(handlers::login_form))
        .route("/login", post(handlers::login_submit))
        .route("/logout", post(handlers::logout))
        .layer(session_layer)
        .with_state(AppState { db: pool , config});

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}