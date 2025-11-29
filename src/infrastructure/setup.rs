use crate::adapters::http::app_state::AppState;
use crate::infrastructure::postgres_persistence;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let postgres_arc = Arc::new(postgres_persistence().await?);

    let beta_applicant_use_cases = BetaApplicantUseCases::new(postgres_arc.clone());

    Ok(AppState {
        beta_applicant_use_cases: Arc::new(beta_applicant_use_cases),
    })
}

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("catalytics_core=info,sqlx=warn"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
