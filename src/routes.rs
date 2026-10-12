use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware as axum_middleware,
    routing::{get, post},
};
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use crate::{handlers, middleware, state::AppState};

pub fn app(state: AppState, session_layer: SessionManagerLayer<PostgresStore>) -> Router {
    // private routes (auth)
    let admin_routes = Router::new()
        .route("/admin/pages/new", get(handlers::new_page_form))
        .route("/admin/pages", post(handlers::create_page))
        .route("/admin/pages/{slug}/qr", get(handlers::generate_qr))
        .route_layer(axum_middleware::from_fn(middleware::require_auth));

    // public routes ( no auth)
    Router::new()
        .route("/admin/media", get(handlers::media_page))
        .route(
            "/admin/media/upload",
            post(handlers::upload_media).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        ) // 50 MB
        .route("/e/{slug}", get(handlers::show_exhibit))
        .route(
            "/login",
            get(handlers::login_form).post(handlers::login_submit),
        )
        .route("/logout", post(handlers::logout))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .merge(admin_routes)
        .layer(session_layer)
        .with_state(state)
}
