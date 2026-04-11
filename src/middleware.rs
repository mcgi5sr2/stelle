use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

pub async fn require_auth(session: Session, request: Request, next: Next) -> Response {
    if session
        .get::<i32>("user_id")
        .await
        .unwrap_or(None)
        .is_some()
    {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}
