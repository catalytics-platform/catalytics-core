use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use axum::extract::FromRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub beta_applicant_use_cases: Arc<BetaApplicantUseCases>,
}

impl FromRef<AppState> for Arc<BetaApplicantUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.beta_applicant_use_cases.clone()
    }
}
