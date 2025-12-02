use crate::app_error::AppResult;
use crate::entities::badge::Badge;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait BadgePersistence: Send + Sync {
    async fn read_badges(&self, public_key: &str) -> AppResult<Vec<Badge>>;
}

#[derive(Clone)]
pub struct BadgeUseCases {
    persistence: Arc<dyn BadgePersistence>,
}

impl BadgeUseCases {
    pub fn new(persistence: Arc<dyn BadgePersistence>) -> Self {
        Self { persistence }
    }

    pub async fn read_all(&self, public_key: &str) -> AppResult<Vec<Badge>> {
        let badges = self.persistence.read_badges(public_key).await?;
        Ok(badges)
    }
}
