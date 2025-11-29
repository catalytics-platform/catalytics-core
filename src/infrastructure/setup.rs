use crate::adapters::http::app_state::AppState;
use crate::infrastructure::postgres_persistence;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use std::sync::Arc;

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let postgres_arc = Arc::new(postgres_persistence().await?);

    let beta_applicant_use_cases = BetaApplicantUseCases::new(postgres_arc.clone());

    Ok(AppState {
        beta_applicant_use_cases: Arc::new(beta_applicant_use_cases)
    })
}

