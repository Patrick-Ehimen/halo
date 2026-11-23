use serde::Serialize;
use specta::Type;

/// Internal error type for the Rust core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database: {0}")]
    Db(#[from] halo_store::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("{0}")]
    Timer(halo_timer::Error),
    #[error("settings: {0}")]
    Json(#[from] serde_json::Error),
    /// Input the user can fix. The message is user-facing.
    #[error("{0}")]
    Invalid(&'static str),
    #[error("state lock poisoned")]
    Poisoned,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Error shape sent over IPC. Typed on the TS side via tauri-specta.
#[derive(Debug, Serialize, Type)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum IpcError {
    Db(String),
    Io(String),
    /// The timer refused an action (e.g. no pauses left). The message is user-facing.
    Timer(String),
    /// Input the user can fix. The message is user-facing.
    Invalid(String),
    Internal(String),
}

impl From<Error> for IpcError {
    fn from(err: Error) -> Self {
        let message = err.to_string();
        match err {
            Error::Db(_) => Self::Db(message),
            Error::Io(_) => Self::Io(message),
            Error::Timer(_) => Self::Timer(message),
            Error::Invalid(_) => Self::Invalid(message),
            Error::Tauri(_) | Error::Json(_) | Error::Poisoned => Self::Internal(message),
        }
    }
}
