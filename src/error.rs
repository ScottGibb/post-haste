#[cfg(target_os = "none")]
pub mod imports {
    pub use embassy_executor::SpawnError;
    pub use embassy_sync::{channel::TrySendError, mutex::TryLockError};
    pub use embassy_time::TimeoutError;
}
#[cfg(not(target_os = "none"))]
pub mod imports {
    pub use tokio::sync::{TryLockError, mpsc::error::SendError, mpsc::error::TrySendError};
}

use imports::*;

#[derive(Debug)]
pub enum PostmasterError {
    AddressAlreadyTaken,
    NoRecipient,
    Timeout,
    TryLockFailed,
    #[cfg(not(target_os = "none"))]
    SendFailed, // Tokio Specific
    TrySendFailed,
    DelayedMessageTaskSpawnFailed,
}

impl From<TryLockError> for PostmasterError {
    fn from(_: TryLockError) -> Self {
        Self::TryLockFailed
    }
}

impl<T> From<TrySendError<T>> for PostmasterError {
    fn from(_: TrySendError<T>) -> Self {
        Self::TrySendFailed
    }
}

#[cfg(target_os = "none")]
impl From<TimeoutError> for PostmasterError {
    fn from(_: TimeoutError) -> Self {
        Self::Timeout
    }
}

#[cfg(target_os = "none")]
impl From<SpawnError> for PostmasterError {
    fn from(_: SpawnError) -> Self {
        Self::DelayedMessageTaskSpawnFailed
    }
}

#[cfg(not(target_os = "none"))]
impl<T> From<SendError<T>> for PostmasterError {
    fn from(_: SendError<T>) -> Self {
        Self::TrySendFailed
    }
}
