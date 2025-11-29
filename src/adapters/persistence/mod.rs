use sqlx::{Error, PgPool};
use crate::app_error::AppError;

pub mod beta_applicant;

#[derive(Clone)]
pub struct PostgresPersistence {
    pool: PgPool,
}

impl PostgresPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: Error) -> Self {
        AppError::Database(value.to_string())
    }
}
