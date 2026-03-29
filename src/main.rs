use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main()]
async fn main() {
    let app = Router::new()
        .route("/", get(hello));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello from Stele!"
}