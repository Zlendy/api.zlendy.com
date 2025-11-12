#[derive(Debug)]
pub enum ResponseError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    ParseIntError(std::num::ParseIntError),
    UnauthorizedError,
    NotFoundError,
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

impl From<std::num::ParseIntError> for ResponseError {
    fn from(error: std::num::ParseIntError) -> Self {
        ResponseError::ParseIntError(error)
    }
}
