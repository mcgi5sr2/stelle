use axum::{
    body::Body,
    extract::{Form, Path, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
};
use qrcode::{QrCode, render::svg};

use crate::models::{NewPage, Page};
use crate::state::AppState;
use crate::slug::slugify;

pub async fn show_exhibit(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Html<String> {
    let page = sqlx::query_as!(Page, "SELECT title, body FROM pages WHERE slug = $1", slug)
        .fetch_one(&state.db)
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

pub async fn new_page_form() -> Html<String> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <h1>New Page</h1>
            <form method="POST" action="/admin/pages">
                <label>Slug<br><input type="text" name="slug"></label><br><br>
                <label>Title<br><input type="text" name="title"></label><br><br>
                <label>Body<br><textarea name="body" rows="10" cols="40"></textarea></label><br><br>
                <button type="submit">Create Page</button>
            </form>
        </body>
        </html>
    "#.to_string())
}

pub async fn create_page(
    State(state): State<AppState>,
    Form(input): Form<NewPage>,
) -> Response {
    let Some(slug) = slugify(&input.slug) else {
        return Html("<h1>Invlaid slug</h1>".to_string()).into_response();
    };

    let result = sqlx::query!(
        "INSERT INTO pages (slug, title, body) VALUES ($1, $2, $3)",
       slug,
        input.title,
        input.body
    )
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Redirect::to(&format!("/e/{}", slug)).into_response(),
        Err(_) => Html("<h1> A page with that slug alread exists</h1>".to_string()).into_response(),
    }
}

pub async fn generate_qr(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Response {
    let url = format!("{}/e/{}", state.config.base_url, slug);
    let code = QrCode::new(url.as_bytes()).unwrap();
    let svg = code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();

    Response::builder()
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}.svg\"", slug),
        )
        .body(Body::from(svg))
        .unwrap()
}
