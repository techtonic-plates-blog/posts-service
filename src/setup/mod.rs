use crate::config;
use crate::connectors::ObjectStorage;
use anyhow::anyhow;
use rdkafka::{ClientConfig, producer::FutureProducer};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use tracing::debug;

mod kafka;

pub fn get_object_storage() -> anyhow::Result<ObjectStorage> {
    Ok(ObjectStorage::new(
        config::CONFIG.minio_url.clone(),
        config::CONFIG.minio_access.clone(),
        config::CONFIG.minio_secret.clone(),
    )?)
}


pub async fn db_init(database_url: &str) -> anyhow::Result<DatabaseConnection> {
    let opt = ConnectOptions::new(database_url.to_string());
    let db = Database::connect(opt).await?;
    Ok(db)
}

pub struct SetupResult {
    pub db: DatabaseConnection,
    pub object_storage: ObjectStorage,
}

pub async fn setup_all() -> anyhow::Result<SetupResult> {
    let db = db_init(&config::CONFIG.database_url).await?;
    kafka::kafka_setup(db.clone()).await?;
    let object_storage = get_object_storage()?;
    Ok(SetupResult { db, object_storage })
}
