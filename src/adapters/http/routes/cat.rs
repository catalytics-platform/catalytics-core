use crate::adapters::http::app_state::AppState;
use crate::app_error::AppResult;
use crate::entities::cat::Cat;
use crate::use_cases::cat::CatUseCases;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
// chrono imports not needed for response DTOs
use serde::Serialize;
use std::sync::Arc;
use tracing::instrument;

pub fn public_router() -> Router<AppState> {
    Router::new().route("/", get(read_cats))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CatResponse {
    id: i32,
    name: String,
    description: String,
    sprite_idle: String,
    sprite_mining: String,
    is_starter: bool,
    levels: Vec<CatLevelResponse>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CatLevelResponse {
    level: i32,
    damage: f64,
    critical_chance: f64,
    critical_multiplier: f64,
    cost: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetCatsResponse {
    cats: Vec<CatResponse>,
}

impl From<Cat> for CatResponse {
    fn from(cat: Cat) -> Self {
        CatResponse {
            id: cat.id,
            name: cat.name,
            description: cat.description,
            sprite_idle: cat.sprite_idle,
            sprite_mining: cat.sprite_mining,
            is_starter: cat.is_starter,
            levels: cat.levels.into_iter().map(CatLevelResponse::from).collect(),
        }
    }
}

impl From<crate::entities::cat::CatLevel> for CatLevelResponse {
    fn from(level: crate::entities::cat::CatLevel) -> Self {
        CatLevelResponse {
            level: level.level,
            damage: level.damage as f64 / 100.0,
            critical_chance: level.critical_chance as f64 / 100.0,
            critical_multiplier: level.critical_multiplier as f64 / 100.0,
            cost: level.cost as f64 / 100.0,
        }
    }
}

#[instrument(skip(use_cases))]
async fn read_cats(State(use_cases): State<Arc<CatUseCases>>) -> AppResult<impl IntoResponse> {
    let cats = use_cases.read_cats().await?;

    let response = GetCatsResponse {
        cats: cats.into_iter().map(CatResponse::from).collect(),
    };

    Ok((StatusCode::OK, Json(response)))
}
