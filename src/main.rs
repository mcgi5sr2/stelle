mod state;
mod models;
mod handlers;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use state::AppState;


#[tokio::main()]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        // slug route this is where user are sent
        .route("/e/{slug}", get(handlers::show_exhibit))
        .route("/admin/pages/new", get(handlers::new_page_form))
        .route("/admin/pages", post(handlers::create_page))
        .route("/admin/pages/{slug}/qr", get(handlers::generate_qr))
        .with_state(AppState { db: pool });

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}