use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Cat {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sprite_idle: String,
    pub sprite_mining: String,
    pub is_starter: bool,
    pub created_at: DateTime<Utc>,
    pub levels: Vec<CatLevel>,
}

#[derive(Debug, Clone)]
pub struct CatLevel {
    pub level: i32,
    pub damage: i64,
    pub critical_chance: i64,
    pub critical_multiplier: i64,
    pub cost: i64,
}
