//! Error type returned by [`crate::data::ClusterData`] implementations.

use std::fmt;

/// A failure while talking to the underlying data source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataError {
    /// The backend (mock store, Prometheus, `slurmrestd`, …) failed.
    Backend(String),
}

impl DataError {
    pub fn backend(msg: impl Into<String>) -> Self {
        DataError::Backend(msg.into())
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Backend(msg) => write!(f, "data backend error: {msg}"),
        }
    }
}

impl std::error::Error for DataError {}
