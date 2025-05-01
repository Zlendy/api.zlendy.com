use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

pub async fn verify(host: String, token: Option<String>) -> Result<String, ResponseError> {
    println!("fn: umami::verify");

    let Some(token) = token else {
        return Err(ResponseError::UnauthorizedError);
    };

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{host}/api/auth/verify"))
        .header("authorization", format!("Bearer {token}"))
        .send()
        .await?
        .status();

    if response.is_success() {
        Ok(token)
    } else {
        Err(ResponseError::UnauthorizedError)
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub user: LoginResponseUser,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LoginResponseUser {
    pub id: String,
    pub username: String,
    pub role: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "isAdmin")]
    pub is_admin: bool,
}

/// Returns API token
pub async fn login(host: String, login: LoginRequest) -> Result<LoginResponse, ResponseError> {
    println!("fn: umami::login");

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{host}/api/auth/login"))
        .body(serde_json::to_string(&login)?)
        .send()
        .await?
        .json::<LoginResponse>()
        .await?;

    println!(
        "info: logged in as {} ({})",
        response.user.username, response.user.role
    );

    Ok(response)
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]

struct DateRange {
    mindate: DateTime<Utc>,
    maxdate: DateTime<Utc>,
}

async fn daterange(
    host: String,
    token: String,
    website_id: String,
) -> Result<DateRange, ResponseError> {
    println!("fn: umami::daterange");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{host}/api/websites/{website_id}/daterange"))
        .header("authorization", format!("Bearer {token}"))
        .send()
        .await?
        .json::<DateRange>()
        .await?;

    Ok(response)
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]

struct StatsResponse {
    pageviews: StatsResponseValue,
    visitors: StatsResponseValue,
    visits: StatsResponseValue,
    bounces: StatsResponseValue,
    totaltime: StatsResponseValue,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]

struct StatsResponseValue {
    value: u64,
    prev: u64,
}

pub async fn pageviews_path(
    host: String,
    token: String,
    website_id: String,
    path: String,
) -> Result<u64, ResponseError> {
    println!("fn: umami::pageviews_path");

    let daterange = daterange(host.clone(), token.clone(), website_id.clone()).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{host}/api/websites/{website_id}/stats"))
        .header("authorization", format!("Bearer {token}"))
        .query(&[("startAt", daterange.mindate.timestamp_millis())])
        .query(&[("endAt", daterange.maxdate.timestamp_millis())])
        .query(&[("url", path)])
        .send()
        .await?
        .json::<StatsResponse>()
        .await?;

    Ok(response.pageviews.value)
}
