use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::badge_group::BadgeGroup;
use crate::use_cases::badge_group::BadgeGroupPersistence;

#[derive(sqlx::FromRow, Debug)]
pub struct BadgeGroupDb {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl PostgresPersistence {
    fn convert_to_badge_groups(&self, badge_groups: Vec<BadgeGroupDb>) -> AppResult<Vec<BadgeGroup>> {
        let result = badge_groups.into_iter().map(|badge_group| BadgeGroup {
            id: badge_group.id,
            title: badge_group.title,
            description: badge_group.description,
            created_at: badge_group.created_at,
        });
        
        Ok(result.collect::<Vec<BadgeGroup>>())
    }
}

#[async_trait]
impl BadgeGroupPersistence for PostgresPersistence {
    async fn read_badge_groups(&self) -> AppResult<Vec<BadgeGroup>> {
        let badge_groups = sqlx::query_as!(BadgeGroupDb, "SELECT * FROM badge_groups b ORDER BY b.id")
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::from)?;
        
        Ok(self.convert_to_badge_groups(badge_groups)?)
    }
}