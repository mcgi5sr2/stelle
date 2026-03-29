use axum::{Router, routing::get, extract::{Path, State}};
use sqlx::{PgPool, FromRow};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[derive(FromRow)]
struct Page  {
    title: String,
    body: Option<String>,
}

#[tokio::main()]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await.expect("Failed to connet to database");

    let app = Router::new()
        .route("/e/{slug}", get(show_exhibit))
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn show_exhibit (
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> String {
    let page = sqlx::query_as!(
        Page,
        "SELECT title, body FROM pages  WHERE slug = $1",
        slug
    )
    .fetch_one(&pool)
    .await;

    match page {
        Ok(p) => format!("<h1>{}</h1><p>{}</p>", p.title, p.body.unwrap_or_default()),
        Err(_) => "Page not found".to_string(),
    }
}