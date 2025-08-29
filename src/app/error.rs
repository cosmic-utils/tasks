use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ron spanned error: {0}")]
    RonSpanned(#[from] ron::error::SpannedError),
    #[error("Ron deserialization error: {0}")]
    RonDeserialization(#[from] ron::de::Error),
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Tasks(#[from] TasksError),
    #[error("{0}")]
    LocalStorage(#[from] LocalStorageError),
}

#[derive(Debug, Error)]
pub enum TasksError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task already exists")]
    ExistingTask,
    #[error("List not found")]
    ListNotFound,
    #[error("List already exists")]
    ExistingList,
    #[error("List ID not found in task")]
    ListIdNotFound,
    #[error("API error  ")]
    ApiError,
}

#[derive(Debug, Error)]
pub enum LocalStorageError {
    #[error("The XDG local directory could not be found.")]
    XdgLocalDirNotFound,
    #[error("The local storage directory could not be created")]
    LocalStorageDirectoryCreationFailed(std::io::Error),
    #[error("The lists directory could not be created")]
    ListsDirectoryCreationFailed(std::io::Error),
    #[error("The tasks directory could not be created")]
    TasksDirectoryCreationFailed(std::io::Error),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

impl LocalStorageError {
    pub fn cause(&self) -> Option<String> {
        match self {
            LocalStorageError::LocalStorageDirectoryCreationFailed(e) => Some(e.to_string()),
            LocalStorageError::ListsDirectoryCreationFailed(e) => Some(e.to_string()),
            LocalStorageError::TasksDirectoryCreationFailed(e) => Some(e.to_string()),
            _ => None,
        }
    }
}
