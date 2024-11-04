use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The requested service was unavailable")]
    ServiceUnavailable,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ron spanned error: {0}")]
    RonSpanned(#[from] ron::error::SpannedError),
    #[error("Ron deserialization error: {0}")]
    RonDeserialization(#[from] ron::de::Error),
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Libset error: {0}")]
    Libset(#[from] libset::Error),
    #[error("TaskS error: {0}")]
    Tasks(#[from] TasksError),
}

#[derive(Debug, Error)]
pub enum TasksError {
    #[error("The requested service is unavailable")]
    ServiceUnavailable,
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task already exists")]
    ExistingTask,
    #[error("List not found")]
    ListNotFound,
    #[error("List already exists")]
    ExistingList,
}
