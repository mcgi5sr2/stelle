use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    extract::{Form, Path, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
};
use tower_sessions::Session;
use qrcode::{QrCode, render::svg};

use crate::error::AppError;
use crate::models::{LoginForm, NewPage, Page};
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
) -> Result<Response, AppError> {
    let url = format!("{}/e/{}", state.config.base_url, slug);
    let code = QrCode::new(url.as_bytes())
        .map_err(|_| AppError::QrCode)?;
    let svg = code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}.svg\"", slug),
        )
        .body(Body::from(svg))
        .unwrap())
}

pub async fn login_form() -> Html<String> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <h1>Login</h1>
            <form method="POST" action="/login">
                <label>Username<br><input type="text" name="username"></label><br><br>
                <label>Password<br><input type="password" name="password"></label><br><br>
                <button type="submit">Login</button>
            </form>
        </body>
        </html>
    "#.to_string())
}

pub async fn login_submit(
    State(state): State<AppState>,
    session: Session,
    Form(input): Form<LoginForm>,
) -> Response {
    println!("1: handler reached");
    
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1 AND deactivated_at IS NULL",
        input.username
    )
    .fetch_optional(&state.db)
    .await
    .expect("Database error");

    println!("2: user found: {}", user.is_some());

    let valid = user.as_ref().map_or(false, |u| {
        let parsed = PasswordHash::new(&u.password_hash).unwrap();
        Argon2::default().verify_password(input.password.as_bytes(), &parsed).is_ok()
    });

    println!("3: valid: {}", valid);

    if valid {
        let user_id = user.unwrap().id;
        println!("4: inserting session for user_id: {}", user_id);
        if let Err(e) = session.insert("user_id", user_id).await {
            println!("Session insert error: {:?}", e);
            return Html("<h1>Session error</h1>".to_string()).into_response();
        }
        println!("5: session inserted");
        Redirect::to("/admin/pages/new").into_response()
    } else {
        Html("<h1>Invalid username or password</h1>".to_string()).into_response()
    }

}

pub async fn logout(session: Session) -> Redirect {
    session.flush().await.unwrap();
    Redirect::to("/login")
}
