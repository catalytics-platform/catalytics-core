use crate::adapters::http::app_state::AppState;
use axum::{http, Router};
use tower_http::cors::CorsLayer;
use crate::adapters;

pub fn create_app(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse().unwrap(),
            "https://axum-websocket-test-1auy.bolt.host/".parse().unwrap(),
        ]
        )
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([http::header::CONTENT_TYPE]);

    Router::new()
        .nest("/api", adapters::http::routes::router())
        .with_state(app_state)
        .layer(cors)

}