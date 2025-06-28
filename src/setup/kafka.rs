use crate::config;
use anyhow::anyhow;
use sea_orm::{ ConnectionTrait, DatabaseConnection};
use tracing::debug;


pub async fn kafka_setup(db: DatabaseConnection) -> anyhow::Result<()> {
    // SQL for Debezium user and publication setup
    const DEBEZIUM_SQL: &str = r#"
        CREATE ROLE debezium_user WITH REPLICATION LOGIN PASSWORD 'debezium_pass';
        GRANT SELECT ON public.posts TO debezium_user;
        CREATE PUBLICATION debezium_posts_pub FOR TABLE public.posts;
        GRANT CREATE ON DATABASE techtonic_plates TO debezium_user;
    "#;

    // Try to set up Debezium user and publication
    if let Err(e) = db.execute_unprepared(DEBEZIUM_SQL).await {
        debug!("Could not set up user: {}", e);
    }

    // Prepare Kafka Connect request
    let client = reqwest::Client::new();
    let url = reqwest::Url::parse(&config::CONFIG.kafka_connect_url)?;
    let db_url = reqwest::Url::parse(&config::CONFIG.database_url)?;

    let connector_config = serde_json::json!({
        "name": "posts-connector",
        "config": {
            "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
            "database.hostname": db_url.host_str().unwrap_or_default(),
            "database.port": db_url.port().unwrap_or(5432),
            "database.user": "debezium_user",
            "database.password": "debezium_pass",
            "database.dbname": "techtonic_plates",
            "topic.prefix": "posts",
            "plugin.name": "pgoutput",
            "slot.name": "debezium_posts_slot",
            "publication.name": "debezium_posts_pub",
            "table.include.list": "public.posts"
        }
    });

    // Send connector creation request
    let res = client
        .post(url.join("connectors")?)
        .json(&connector_config)
        .send()
        .await?;

    // Accept 200 OK or 409 Conflict (already exists)
    if !matches!(res.status().as_u16(), 200 | 201 | 409) {
        return Err(anyhow!("{}", res.text().await?));
    }

    Ok(())
}