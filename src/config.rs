#[derive(Clone)]
pub struct Config {
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            base_url: std::env::var("BASE_URL").expect("BASE_URL must be set"),
        }
    }
}
