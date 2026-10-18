use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cosmic config error: {0}")]
    CosmicConfig(#[from] cosmic::cosmic_config::Error),

    #[error("Deserialization error: {0}")]
    Deserialize(#[from] ron::error::SpannedError),

    #[error("Iced error: {0}")]
    Iced(#[from] cosmic::iced::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Serialization error: {0}")]
    Serialize(#[from] ron::Error),

    #[error("{0}")]
    Store(#[from] StoreError),
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("List not found: {0}")]
    ListNotFound(uuid::Uuid),

    #[error("State not found: {0}")]
    StateNotFound(uuid::Uuid),

    #[error("Task not found: {0}")]
    TaskNotFound(uuid::Uuid),
}

pub type Result<T> = std::result::Result<T, Error>;
