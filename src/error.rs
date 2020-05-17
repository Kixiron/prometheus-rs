use std::{error::Error, fmt};

pub type Result<T> = std::result::Result<T, PromError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromError {
    message: String,
    kind: PromErrorKind,
}

impl PromError {
    pub(crate) fn new(message: impl Into<String>, kind: PromErrorKind) -> Self {
        Self {
            message: message.into(),
            kind,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn kind(&self) -> PromErrorKind {
        self.kind
    }
}

impl fmt::Display for PromError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prometheus Error ({:?}): {}", self.kind, self.message)
    }
}

impl Error for PromError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PromErrorKind {
    IncrementNegative,
    InvalidLabelName,
    InvalidMetricName,
    MissingComponent,
    BucketNotFound,
}
