use crate::app_error::AppResult;
use crate::entities::leaderboard_entry::{LeaderboardEntry, LeaderboardEntryDto};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait LeaderboardPersistence: Send + Sync + Debug {
    async fn get_leaderboard_entries(
        &self,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<LeaderboardEntry>>;
    async fn get_total_users_with_badges(&self) -> AppResult<u32>;
    async fn get_user_rank(&self, public_key: &str) -> AppResult<u32>;
    async fn get_user_leaderboard_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntry>>;
    async fn get_user_realtime_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntry>>;
    async fn add_new_user_to_leaderboard(
        &self,
        beta_applicant_id: i32,
        public_key: &str,
    ) -> AppResult<()>;
}

#[derive(Clone, Debug)]
pub struct LeaderboardUseCases {
    persistence: Arc<dyn LeaderboardPersistence>,
}

impl LeaderboardUseCases {
    pub fn new(persistence: Arc<dyn LeaderboardPersistence>) -> Self {
        Self { persistence }
    }

    pub async fn get_leaderboard(
        &self,
        page: u32,
        limit: u32,
    ) -> AppResult<(Vec<LeaderboardEntryDto>, u32)> {
        let offset = (page - 1) * limit;
        let entries = self
            .persistence
            .get_leaderboard_entries(limit, offset)
            .await?;
        let total = self.persistence.get_total_users_with_badges().await?;

        let dto_entries = entries.into_iter().map(LeaderboardEntryDto::from).collect();
        Ok((dto_entries, total))
    }

    pub async fn get_user_rank(&self, public_key: &str) -> AppResult<u32> {
        self.persistence.get_user_rank(public_key).await
    }

    pub async fn get_user_leaderboard_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntryDto>> {
        let entry = self
            .persistence
            .get_user_leaderboard_entry(public_key)
            .await?;
        Ok(entry.map(LeaderboardEntryDto::from))
    }

    pub async fn get_user_realtime_leaderboard_entry(
        &self,
        public_key: &str,
    ) -> AppResult<Option<LeaderboardEntryDto>> {
        let entry = self.persistence.get_user_realtime_entry(public_key).await?;
        Ok(entry.map(LeaderboardEntryDto::from))
    }

    pub async fn add_new_user_to_leaderboard(
        &self,
        beta_applicant_id: i32,
        public_key: &str,
    ) -> AppResult<()> {
        self.persistence
            .add_new_user_to_leaderboard(beta_applicant_id, public_key)
            .await
    }
}
