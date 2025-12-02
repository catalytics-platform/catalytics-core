use crate::app_error::AppError;
use sqlx::{Error, PgPool};

mod badge;
mod beta_applicant;
mod beta_applicant_badge;

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
        match value {
            sqlx::Error::RowNotFound => AppError::NotFound("Resource not found".to_string()),
            _ => AppError::Database(value.to_string()),
        }
    }
}
