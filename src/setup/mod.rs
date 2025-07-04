use crate::config;
use crate::connections::ObjectStorage;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

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
    let object_storage = get_object_storage()?;
    Ok(SetupResult { db, object_storage })
}
