use std::{fmt::Display, path::Path, str::FromStr};
use thiserror::Error;
use url::Url;

#[derive(Error, Clone, Debug)]
pub enum LaneError {
    #[error("Config file {0} doesn't exist or content error.")]
    InvalidFile(String),
    #[error("Failed writing config file {0}.")]
    WriteFailure(String),
    #[error("Invalid proxy url: {0}.")]
    InvalidProxyUrl(String),
    #[error("Nothing to do.")]
    NothingToDo(),
    #[error("Cannot get home dir.")]
    NoHomeDir(),
    #[error("Unknown mirror {0}")]
    UnknownMirror(String),
    #[error("Failed. Reason: {0}")]
    Failure(String),
}

pub fn make_invalid_file_error(path: &Path) -> LaneError {
    LaneError::InvalidFile(path.to_str().unwrap_or_default().to_string())
}

pub fn make_write_file_error(path: &Path) -> LaneError {
    LaneError::WriteFailure(path.to_str().unwrap_or_default().to_string())
}

pub fn make_unknown_mirror_error(mirror: &str) -> LaneError {
    LaneError::UnknownMirror(mirror.to_string())
}

pub fn make_failure_error(reason: impl Display) -> LaneError {
    LaneError::Failure(reason.to_string())
}

pub fn validate_proxy_url(url: &str) -> Result<(), LaneError> {
    let _ = Url::from_str(url).map_err(|_| LaneError::InvalidProxyUrl(url.to_string()))?;
    Ok(())
}
