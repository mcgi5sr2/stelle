use axum::{
    body::Body,
    extract::{Form, Path, State},
    http::header,
    Router,
    routing::{get, post},
    response::{Html, Redirect, Response},
};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;
use qrcode::{QrCode, render::svg};

#[derive(FromRow)]
struct Page {
    title: String,
    body: Option<String>,
}

#[derive(serde::Deserialize)]
struct NewPage {
    slug: String,
    title: String,
    body: String,
}

#[tokio::main()]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        // slug route this is where user are sent
        .route("/e/{slug}", get(show_exhibit))
        .route("/admin/pages/new", get(new_page_form))
        .route("/admin/pages", post(create_page))
        .route("/admin/pages/{slug}/qr", get(generate_qr))
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn show_exhibit(State(pool): State<PgPool>, Path(slug): Path<String>) -> Html<String> {
    let page = sqlx::query_as!(Page, "SELECT title, body FROM pages WHERE slug = $1", slug)
        .fetch_one(&pool)
        .await;

    match page {
        Ok(p) => Html(format!(
            "<h1>{}</h1><p>{}</p>",
            p.title,
            p.body.unwrap_or_default()
        )),
        Err(_) => Html("<h1>Page not found</h1>".to_string()),
    }
}

async fn new_page_form() -> Html<String> {
    Html(
        r#"
    <!DOCTPYE html>
    <html>
    <body>
        <h1> New Page</h1>
        <form method="POST" action="/admin/pages">
            <label>Slug<br><input type="test" name="slug"></label><br><br>
            <lable>Title<br><input type="text" name="title"></label><br><br>
            <label>Body<br><textarea name="body" rows="10" cols="40"></textarea></label><br><br>
            <button type="submint"> Create Page</button>
        </form>
    </body>
    </html>
    "#
        .to_string(),
    )
}

async fn create_page(State(pool): State<PgPool>, Form(input): Form<NewPage>) -> Redirect {
    sqlx::query!(
        "INSERT INTO pages (slug, title, body) VALUES ($1, $2, $3)",
        input.slug,
        input.title,
        input.body
    )
    .execute(&pool)
    .await
    .expect("Failed to insert page");

    Redirect::to(&format!("/e/{}", input.slug))
}

async fn generate_qr(Path(slug): Path<String>) -> Response {
    let url = format!("http://localhost:3000/e/{}", slug);
    let code  = QrCode::new(url.as_bytes()).unwrap();
    let svg = code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();

    Response::builder()
        .header(header::CONTENT_TYPE, "images/svg+xml")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.svg\"", slug),
        )
        .body(Body::from(svg))
        .unwrap()
}