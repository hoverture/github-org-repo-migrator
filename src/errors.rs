use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoCopyError {
    #[error("HTTP error: {0}")]
    Http(reqwest::Error),

    #[error("HTTP error: {0}")]
    HttpError(reqwest::StatusCode),

    #[error("Git error: {0}")]
    Git(git2::Error),

    #[error("I/O error: {0}")]
    Io(std::io::Error),

    #[error("JSON error: {0}")]
    Json(serde_json::Error),

    #[error("Header error: {0}")]
    InvalidHeader(reqwest::header::InvalidHeaderValue),

    #[error("CreateRepo error: {0}")]
    CreateRepoError(reqwest::StatusCode),

    #[error("SetTopics error: {0}")]
    SetTopicsError(reqwest::StatusCode),
}

impl From<reqwest::Error> for RepoCopyError {
    fn from(err: reqwest::Error) -> RepoCopyError {
        RepoCopyError::Http(err)
    }
}

impl From<reqwest::StatusCode> for RepoCopyError {
    fn from(status_code: reqwest::StatusCode) -> Self {
        match status_code {
            _ if status_code.is_client_error() => RepoCopyError::HttpError(status_code),
            _ if status_code.is_server_error() => RepoCopyError::HttpError(status_code),
            _ => RepoCopyError::HttpError(status_code),
        }
    }
}

impl From<git2::Error> for RepoCopyError {
    fn from(err: git2::Error) -> RepoCopyError {
        RepoCopyError::Git(err)
    }
}

impl From<std::io::Error> for RepoCopyError {
    fn from(err: std::io::Error) -> RepoCopyError {
        RepoCopyError::Io(err)
    }
}

impl From<InvalidHeaderValue> for RepoCopyError {
    fn from(err: InvalidHeaderValue) -> RepoCopyError {
        RepoCopyError::InvalidHeader(err)
    }
}

impl From<serde_json::Error> for RepoCopyError {
    fn from(err: serde_json::Error) -> RepoCopyError {
        RepoCopyError::Json(err)
    }
}
