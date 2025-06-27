use once_cell::sync::Lazy;
use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub minio_url: String,
    pub minio_access: String,
    pub minio_secret: String,
    pub kafka_connect_url: String,
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| AppConfig {
    database_url: env::var("DATABASE_URL").expect("Could not get database url"),
    minio_url: env::var("MINIO_URL").expect("Could not get minio url"),
    minio_access: env::var("MINIO_ACCESS").expect("Could not get minio access key"),
    minio_secret: env::var("MINIO_SECRET").expect("Could not get minio secret key"),
    kafka_connect_url: env::var("KAFKA_CONNECT_URL").expect("Kafka connect url not set up"),
});
