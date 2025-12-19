use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use crate::app_error::AppResult;
use crate::entities::badge_group::BadgeGroup;

#[async_trait]
pub trait BadgeGroupPersistence: Send + Sync + Debug {
    async fn read_badge_groups(&self) -> AppResult<Vec<BadgeGroup>>;
}

#[derive(Clone, Debug)]
pub struct BadgeGroupUseCases {
    persistence: Arc<dyn BadgeGroupPersistence>
}

impl BadgeGroupUseCases {
    pub fn new(persistence: Arc<dyn BadgeGroupPersistence>) -> Self {
        Self { persistence }
    }
    
    pub async fn read_all(&self) -> AppResult<Vec<BadgeGroup>> {
        let badge_groups = self.persistence.read_badge_groups().await?;
        Ok(badge_groups)
    }
}