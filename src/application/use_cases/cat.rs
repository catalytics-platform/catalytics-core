use crate::app_error::AppResult;
use crate::entities::cat::Cat;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait CatPersistence: Send + Sync + Debug {
    async fn read_cats(&self) -> AppResult<Vec<Cat>>;
}

#[derive(Clone, Debug)]
pub struct CatUseCases {
    persistence: Arc<dyn CatPersistence>,
}

impl CatUseCases {
    pub fn new(persistence: Arc<dyn CatPersistence>) -> Self {
        Self { persistence }
    }

    pub async fn read_cats(&self) -> AppResult<Vec<Cat>> {
        self.persistence.read_cats().await
    }
}
