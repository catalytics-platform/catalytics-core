use crate::adapters::persistence::PostgresPersistence;
use crate::app_error::{AppError, AppResult};
use crate::entities::cat::{Cat, CatLevel};
use crate::use_cases::cat::CatPersistence;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(sqlx::FromRow, Debug)]
pub struct CatDb {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sprite_idle: String,
    pub sprite_mining: String,
    pub is_starter: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct CatLevelDb {
    pub cat_id: i32,
    pub level: i32,
    pub damage: i64,
    pub critical_chance: i64,
    pub critical_multiplier: i64,
    pub cost: i64,
}

impl PostgresPersistence {
    fn convert_to_cats(
        &self,
        cats: Vec<CatDb>,
        cat_levels: Vec<CatLevelDb>,
    ) -> AppResult<Vec<Cat>> {
        // Group levels by cat_id
        let mut levels_by_cat: HashMap<i32, Vec<CatLevel>> = HashMap::new();

        for level_db in cat_levels {
            let cat_level = CatLevel {
                level: level_db.level,
                damage: level_db.damage,
                critical_chance: level_db.critical_chance,
                critical_multiplier: level_db.critical_multiplier,
                cost: level_db.cost,
            };

            levels_by_cat
                .entry(level_db.cat_id)
                .or_insert_with(Vec::new)
                .push(cat_level);
        }

        // Sort levels by level number
        for levels in levels_by_cat.values_mut() {
            levels.sort_by_key(|l| l.level);
        }

        // Convert cats with their levels
        let result = cats
            .into_iter()
            .map(|cat_db| Cat {
                id: cat_db.id,
                name: cat_db.name,
                description: cat_db.description,
                sprite_idle: cat_db.sprite_idle,
                sprite_mining: cat_db.sprite_mining,
                is_starter: cat_db.is_starter,
                created_at: cat_db.created_at,
                levels: levels_by_cat.get(&cat_db.id).cloned().unwrap_or_default(),
            })
            .collect();

        Ok(result)
    }
}

#[async_trait]
impl CatPersistence for PostgresPersistence {
    async fn read_cats(&self) -> AppResult<Vec<Cat>> {
        let cats = sqlx::query_as!(
            CatDb,
            "SELECT id, name, description, sprite_idle, sprite_mining, is_starter, created_at 
             FROM cats 
             ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        let cat_levels = sqlx::query_as!(
            CatLevelDb,
            "SELECT cat_id, level, damage, critical_chance, critical_multiplier, cost 
             FROM cat_levels 
             ORDER BY cat_id, level"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        self.convert_to_cats(cats, cat_levels)
    }
}
