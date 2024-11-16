use thiserror::Error;

pub type LBTrackerResult<T> = Result<T, LBTrackerError>;

#[derive(Debug, Error)]
pub enum LBTrackerError {
    #[error("Cannot parse node filter: {0}")]
    InvalidNodeFilter(String),
    #[error("Unsupported service type")]
    UnsupportedServiceType,
    #[error("Service was skipped")]
    SkipService,
    #[error("Cannot parse integer value: {0}")]
    PaseIntError(#[from] std::num::ParseIntError),
    #[error("Cannot parse boolean value: {0}")]
    PaseBoolError(#[from] std::str::ParseBoolError),
    #[error("HCloud error: {0}")]
    HCloudError(String),
    #[error("Kube error: {0}")]
    KubeError(#[from] kube::Error),
}

impl<T> From<hcloud::apis::Error<T>> for LBTrackerError {
    fn from(value: hcloud::apis::Error<T>) -> Self {
        Self::HCloudError(format!("{value}"))
    }
}
