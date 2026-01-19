use crate::adapters::http::app_state::AppState;
use crate::infrastructure::{mailchimp_client, postgres_persistence, wallet_holding_client};
use crate::use_cases::badge::BadgeUseCases;
use crate::use_cases::badge_group::BadgeGroupUseCases;
use crate::use_cases::beta_applicant::BetaApplicantUseCases;
use crate::use_cases::beta_applicant_progression::BetaApplicantProgressionUseCases;
use crate::use_cases::leaderboard::LeaderboardUseCases;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let postgres_arc = Arc::new(postgres_persistence().await?);
    let wallet_holding_arc = Arc::new(wallet_holding_client().await?);
    let mailchimp_arc = Arc::new(mailchimp_client().await?);

    let beta_applicant_use_cases = BetaApplicantUseCases::new(postgres_arc.clone(), mailchimp_arc);
    let badge_use_case = BadgeUseCases::new(postgres_arc.clone());
    let badge_group_use_case = BadgeGroupUseCases::new(postgres_arc.clone());
    let beta_applicant_progression_use_cases = BetaApplicantProgressionUseCases::new(
        postgres_arc.clone(),
        postgres_arc.clone(),
        wallet_holding_arc.clone(),
    );
    let leaderboard_use_cases = LeaderboardUseCases::new(postgres_arc.clone());

    Ok(AppState {
        beta_applicant_use_cases: Arc::new(beta_applicant_use_cases),
        badge_use_cases: Arc::new(badge_use_case),
        badge_group_use_cases: Arc::new(badge_group_use_case),
        beta_applicant_progression_use_cases: Arc::new(beta_applicant_progression_use_cases),
        leaderboard_use_cases: Arc::new(leaderboard_use_cases),
    })
}

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("catalytics_core=info,sqlx=warn"));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
