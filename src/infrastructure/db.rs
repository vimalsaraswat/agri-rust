use mongodb::{
    bson::doc,
    options::{ClientOptions, IndexOptions},
    Client, Collection, IndexModel,
};
use tracing::info;

use crate::config::AppConfig;

pub async fn connect(config: &AppConfig) -> Client {
    let mut opts = ClientOptions::parse(&config.database_url)
        .await
        .expect("Failed to parse MongoDB connection string");

    opts.app_name = Some("kishan-mitra".into());

    let client = Client::with_options(opts).expect("Failed to create MongoDB client");

    client
        .database("admin")
        .run_command(doc! { "ping": 1 })
        .await
        .expect("Failed to ping MongoDB — is the server running?");

    info!("Connected to MongoDB");

    create_indexes(&client, &config.database_name).await;

    client
}

async fn create_indexes(client: &Client, db_name: &str) {
    let col: Collection<mongodb::bson::Document> = client.database(db_name).collection("users");
    let unique = IndexOptions::builder().unique(true).build();

    let indexes = vec![
        IndexModel::builder().keys(doc! { "email": 1 }).options(unique.clone()).build(),
        IndexModel::builder().keys(doc! { "phone_number": 1 }).options(unique).build(),
    ];

    col.create_indexes(indexes)
        .await
        .expect("Failed to create MongoDB indexes");

    info!("MongoDB indexes verified");
}
