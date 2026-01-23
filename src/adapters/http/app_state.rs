use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::badge_group::BadgeGroupUseCases;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionUseCases;
use crate::use_cases::cat::CatUseCases;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub beta_applicant_use_cases: Arc<BetaApplicantUseCases>,
    pub badge_use_cases: Arc<BadgeUseCases>,
    pub badge_group_use_cases: Arc<BadgeGroupUseCases>,
    pub beta_applicant_progression_use_cases: Arc<BetaApplicantProgressionUseCases>,
    pub cat_use_cases: Arc<CatUseCases>,
    pub leaderboard_use_cases: Arc<LeaderboardUseCases>,
}

impl FromRef<AppState> for Arc<BetaApplicantUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.beta_applicant_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<BadgeUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.badge_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<BadgeGroupUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.badge_group_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<BetaApplicantProgressionUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.beta_applicant_progression_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<CatUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.cat_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<LeaderboardUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.leaderboard_use_cases.clone()
    }
}
