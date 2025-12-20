mod badge;
mod beta_applicant;
mod health;
mod leaderboard;

use crate::adapters::http::app_state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/beta-applicants", beta_applicant::private_router())
        .nest("/beta-applicants", beta_applicant::public_router())
        .nest("/badges", badge::private_router())
        .nest("/badges", badge::public_router())
        .nest("/leaderboard", leaderboard::public_router())
        .nest("/k8s", health::router())
}
