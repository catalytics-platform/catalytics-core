mod beta_applicant;

use axum::Router;
use crate::adapters::http::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().nest("/beta-applicants", beta_applicant::router())
}