use crate::adapters::http::app_state::AppState;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "catalytics-core",
        "timestamp": chrono::Utc::now()
    }))
}

async fn ready() -> Json<Value> {
    Json(json!({
        "status": "ready",
        "service": "catalytics-core",
        "timestamp": chrono::Utc::now()
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}
