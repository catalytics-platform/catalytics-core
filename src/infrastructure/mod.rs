use crate::adapters::persistence::PostgresPersistence;
use crate::infrastructure::database::init_db;

pub mod database;
pub mod setup;
pub mod app;

pub async fn postgres_persistence() -> anyhow::Result<PostgresPersistence> {
    let pool = init_db().await?;
    let persistence = PostgresPersistence::new(pool);
    Ok(persistence)
}