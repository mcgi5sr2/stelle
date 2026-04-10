use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    extract::{Form, Multipart, Path, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
};
use tower_sessions::Session;
use qrcode::{QrCode, render::svg};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{LoginForm, MediaKind, NewPage, Page};
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

pub async  fn upload_media(
    State(state): State<AppState>,
    mut multipart: Multipart,
 ) -> Result<Response, AppError> {
        while let Some(field) = multipart.next_field().await? {
            let filename = field.file_name()
                .unwrap_or("unknown")
                .to_string();

            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

        let kind = match content_type.as_str() {
            ct if ct.starts_with("image/") => MediaKind::Image,
            ct if ct.starts_with("audio/") => MediaKind::Audio,
            ct if ct.starts_with("video/") => MediaKind::Video,
            "application/pdf" => MediaKind::Pdf,
            _ => return Ok(Html("<h1>Unsupported file type</h1>".to_string()).into_response()),
        };
 
            let data = field.bytes().await?;
            let file_size = data.len() as i64;

            let ext = filename.rsplit('.').next().unwrap_or("bin");
            let storage_path = format!("uploads/{}.{}", Uuid::new_v4(), ext);

            tokio::fs::write(&storage_path, &data).await
                .map_err(|_| AppError::Upload)?;

        sqlx::query!(
            "INSERT INTO media (kind, filename, storage_path, mime_type, file_size, uploaded_by)
             VALUES ($1, $2, $3, $4, $5, $6)",
            kind as MediaKind,
            filename,
            storage_path,
            content_type,
            file_size,
            None::<i32>
        )
            .execute(&state.db)
            .await?;
        }
        Ok(Redirect::to("/admin/media").into_response())
}

pub async fn media_page(State(state): State<AppState>) -> Html<String> {
    let media = sqlx::query!("SELECT id, kind as \"kind: MediaKind\", filename, created_at FROM media ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .expect("Failed to fetch media");

    let rows = media.iter().map(|m| {
        format!("<tr><td>{}</td><td>{:?}</td><td>{}</td></tr>",
            m.id, m.kind, m.filename)
    }).collect::<String>();

    Html(format!(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <h1>Upload Media</h1>
            <form method="POST" action="/admin/media/upload" enctype="multipart/form-data">
                <input type="file" name="file"><br><br>
                <button type="submit">Upload</button>
            </form>
            <h2>Uploaded Files</h2>
            <table border="1">
                <tr><th>ID</th><th>Kind</th><th>Filename</th></tr>
                {}
            </table>
        </body>
        </html>
    "#, rows))
}
