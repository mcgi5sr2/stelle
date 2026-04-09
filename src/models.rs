use sqlx::FromRow;

#[derive(FromRow)]
pub struct Page {
    pub title: String,
    pub body: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct NewPage {
    pub slug: String,
    pub title: String,
    pub body: String,
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "media_kind", rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Audio,
    Video,
    Pdf,
}