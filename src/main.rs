mod api;
mod application;
mod common;
mod config;
mod domain;
mod infrastructure;

use std::{net::SocketAddr, sync::Arc};

use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    api::router::{create_router, AppState},
    application::{auth::AuthService, chat::ChatService, msp::MspService, schemes::SchemesService},
    config::{AppConfig, Environment},
    infrastructure::{
        db, GeminiClient, MongoMspRepository, MongoSchemesRepo, MongoUserRepository, NewsFetcher,
    },
};

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let config = Arc::new(AppConfig::from_env());

    init_tracing(&config);

    info!(version = env!("CARGO_PKG_VERSION"), environment = ?config.environment, "Starting Kishan Mitra");

    let client = db::connect(&config).await;
    let database = client.database(&config.database_name);

    let user_repo = Arc::new(MongoUserRepository::new(&database));
    let auth_service = AuthService::new(Arc::clone(&user_repo), Arc::clone(&config));

    let msp_repo = MongoMspRepository::new(&database);
    msp_repo.ensure_ready().await;
    let msp_service = MspService::new(msp_repo);

    let gemini = GeminiClient::new(config.gemini_api_key.clone());
    let chat_service = ChatService::new(Arc::clone(&user_repo), gemini);

    let schemes_repo = MongoSchemesRepo::new(&database);
    schemes_repo.ensure_ready().await;
    let schemes_service = SchemesService::new(schemes_repo, NewsFetcher::new());

    let state = AppState::new(
        auth_service,
        msp_service,
        chat_service,
        schemes_service,
        Arc::clone(&config),
    );

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port)
        .parse()
        .expect("Invalid server address");

    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Bind failed: {e}"));

    info!(address = %addr, "Server listening");

    axum::serve(listener, create_router(state))
        .await
        .expect("Server error");
}

fn init_tracing(config: &AppConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,agri_rust=debug"));

    let reg = tracing_subscriber::registry().with(filter);

    if config.environment == Environment::Production {
        reg.with(fmt::layer().json()).init();
    } else {
        reg.with(fmt::layer().pretty()).init();
    }
}
