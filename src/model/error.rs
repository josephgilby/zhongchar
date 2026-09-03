use std::{sync::Arc};
use indexed_db_futures::error::OpenDbError;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ArcZhongCharError {
    #[error("{0}")]
    Error(Arc<ZhongCharError>),

}

impl From<ZhongCharError> for ArcZhongCharError {
    fn from(error: ZhongCharError) -> Self {
        ArcZhongCharError::Error(Arc::new(error))
    }
}

pub type ZhongCharResult<T> = Result<T, ZhongCharError>;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ZhongCharError {
    #[error("{0}")]
    Wasm(String),
    #[error("{0}")]
    Reqwasm(#[from] reqwasm::Error),
    #[error("{0}")]
    Csv(#[from] csv::Error),
    #[error("Database Error: {0}")]
    Database(String),
}

impl From<serde_wasm_bindgen::Error> for ZhongCharError {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        ZhongCharError::Wasm(error.to_string()) // Convert to String
    }
}

impl From<OpenDbError> for ZhongCharError {
    fn from(error: OpenDbError) -> Self {
        ZhongCharError::Database(error.to_string())
    }
}

impl From<indexed_db_futures::error::Error> for ZhongCharError {
    fn from(error: indexed_db_futures::error::Error) -> Self {
        ZhongCharError::Database(error.to_string())
    }
}
