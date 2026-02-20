use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Iced error: {0}")]
    Iced(#[from] cosmic::iced::Error),

    #[error("Serialization error: {0}")]
    Serialize(#[from] ron::Error),

    #[error("Deserialization error: {0}")]
    Deserialize(#[from] ron::error::SpannedError),

    #[error("List not found: {0}")]
    ListNotFound(uuid::Uuid),

    #[error("Task not found: {0}")]
    TaskNotFound(uuid::Uuid),
}

pub type Result<T> = std::result::Result<T, AppError>;
