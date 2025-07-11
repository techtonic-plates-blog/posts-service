use once_cell::sync::Lazy;
use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub jwt_public_key: String,
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| AppConfig {
    database_url: env::var("DATABASE_URL").expect("Could not get database url"),
    jwt_public_key: env::var("JWT_PUBLIC_KEY")
        .expect("JWT public key not set")
        .replace("\\n", "\n"),
});
