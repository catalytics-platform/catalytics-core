use crate::adapters::http::app_state::AppState;
use crate::infrastructure::{postgres_persistence, wallet_holding_client};
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let postgres_arc = Arc::new(postgres_persistence().await?);
    let wallet_holding_arc = Arc::new(wallet_holding_client().await?);

    let beta_applicant_use_cases = BetaApplicantUseCases::new(postgres_arc.clone());
    let badge_use_case = BadgeUseCases::new(postgres_arc.clone(), wallet_holding_arc.clone());

    Ok(AppState {
        beta_applicant_use_cases: Arc::new(beta_applicant_use_cases),
        badge_use_cases: Arc::new(badge_use_case),
    })
}

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("catalytics_core=info,sqlx=warn"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
