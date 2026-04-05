use std::sync::Arc;

use axum::{
    http::{HeaderName, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer,
};

use crate::{
    api::handlers,
    application::{auth::AuthService, chat::ChatService, msp::MspService, schemes::SchemesService},
    config::{AppConfig, Environment},
    infrastructure::{MongoMspRepository, MongoUserRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService<MongoUserRepository>>,
    pub msp_service: Arc<MspService<MongoMspRepository>>,
    pub chat_service: Arc<ChatService<MongoUserRepository>>,
    pub schemes_service: Arc<SchemesService>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(
        auth_service: AuthService<MongoUserRepository>,
        msp_service: MspService<MongoMspRepository>,
        chat_service: ChatService<MongoUserRepository>,
        schemes_service: SchemesService,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            auth_service: Arc::new(auth_service),
            msp_service: Arc::new(msp_service),
            chat_service: Arc::new(chat_service),
            schemes_service: Arc::new(schemes_service),
            config,
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let auth = Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me).patch(handlers::update_me));

    let schemes = Router::new()
        .route("/", get(handlers::schemes))
        .route("/news", get(handlers::schemes_news));

    let api_v1 = Router::new()
        .nest("/auth", auth)
        .nest("/schemes", schemes)
        .route("/msp", get(handlers::msp_prices))
        .route("/chat", post(handlers::chat))
        .route("/health", get(handlers::health));

    let x_req_id = HeaderName::from_static("x-request-id");

    Router::new()
        .nest("/api/v1", api_v1)
        .layer(SetSensitiveHeadersLayer::new([axum::http::header::AUTHORIZATION]))
        .layer(PropagateRequestIdLayer::new(x_req_id.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::new(x_req_id, MakeRequestUuid))
        .layer(build_cors(&state.config))
        .with_state(state)
}

fn build_cors(config: &AppConfig) -> CorsLayer {
    if config.environment == Environment::Development {
        return CorsLayer::permissive();
    }

    let origins: Vec<HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(AllowMethods::list([
            Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]))
        .allow_credentials(true)
}
