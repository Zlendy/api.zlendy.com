#[derive(Debug)]
pub enum ResponseError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    UnauthorizedError,
}

impl From<reqwest::Error> for ResponseError {
    fn from(error: reqwest::Error) -> Self {
        ResponseError::ReqwestError(error)
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(error: serde_json::Error) -> Self {
        ResponseError::SerdeJsonError(error)
    }
}
