use std::sync::Arc;
use axum::extract::FromRef;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;

#[derive(Clone)]
pub struct AppState {
    pub beta_applicant_use_cases: Arc<BetaApplicantUseCases>
}

impl FromRef<AppState> for Arc<BetaApplicantUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.beta_applicant_use_cases.clone()
    }
}