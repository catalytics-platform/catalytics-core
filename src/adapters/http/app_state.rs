use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use axum::extract::FromRef;
use std::sync::Arc;
use crate::use_cases::badge_group::BadgeGroupUseCases;

#[derive(Clone)]
pub struct AppState {
    pub beta_applicant_use_cases: Arc<BetaApplicantUseCases>,
    pub badge_use_cases: Arc<BadgeUseCases>,
    pub badge_group_use_cases: Arc<BadgeGroupUseCases>,
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
