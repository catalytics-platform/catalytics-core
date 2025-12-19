use crate::app_error::AppResult;
use crate::entities::badge::{Badge, BadgeDto};
use crate::entities::badge_requirement::{BadgeRequirement, BadgeRequirementDto};
use crate::entities::progression_event_type::ProgressionEventType;

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait BadgePersistence: Send + Sync + Debug {
    async fn read_badges(&self, public_key: &str) -> AppResult<Vec<Badge>>;
    async fn read_badge_requirements(&self) -> AppResult<Vec<BadgeRequirement>>;
    async fn create_badge(&self, public_key: &str, badge_id: i32, value: i32) -> AppResult<()>;

    async fn award_badge_if_eligible(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
        progress_count: i32,
    ) -> AppResult<()>;
}

#[derive(Clone, Debug)]
pub struct BadgeUseCases {
    persistence: Arc<dyn BadgePersistence>,
}

impl BadgeUseCases {
    pub fn new(persistence: Arc<dyn BadgePersistence>) -> Self {
        Self { persistence }
    }

    pub async fn read_all(&self, public_key: &str) -> AppResult<Vec<BadgeDto>> {
        let badges = self.persistence.read_badges(public_key).await?;
        Ok(badges.into_iter().map(BadgeDto::from).collect())
    }

    pub async fn read_badge_requirements(&self) -> AppResult<Vec<BadgeRequirementDto>> {
        let requirements = self.persistence.read_badge_requirements().await?;
        Ok(requirements
            .into_iter()
            .map(BadgeRequirementDto::from)
            .collect())
    }

    pub async fn award_badge_if_eligible(
        &self,
        public_key: &str,
        event_type: ProgressionEventType,
        progress_count: i32,
    ) -> AppResult<()> {
        self.persistence
            .award_badge_if_eligible(public_key, event_type, progress_count)
            .await
    }

    pub async fn create_badge(&self, public_key: &str, badge_id: i32, value: i32) -> AppResult<()> {
        Ok(self
            .persistence
            .create_badge(public_key, badge_id, value)
            .await?)
    }
}
