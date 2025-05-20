use embassy_time::TimeoutError;

pub enum PostmasterError {
    AddressAlreadyTaken,
    NoRecipient,
    Timeout,
}

impl From<TimeoutError> for PostmasterError {
    fn from(_: TimeoutError) -> Self {
        Self::Timeout
    }
}
