use crate::adapters;
use crate::adapters::http::app_state::AppState;
use crate::adapters::http::middleware::auth_middleware;
use crate::infrastructure::setup::init_tracing;
use axum::{Router, http, middleware};
use tower_http::cors::{Any, CorsLayer};

pub fn create_app(app_state: AppState) -> Router {
    init_tracing();

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:4200".parse().unwrap()])
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::PATCH])
        .allow_headers(Any);

    Router::new()
        .nest("/api", adapters::http::routes::router())
        .with_state(app_state)
        .layer(middleware::from_fn(auth_middleware))
        .layer(cors)
}
