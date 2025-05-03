use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::errors::ResponseError;

pub async fn verify(host: String, token: Option<String>) -> Result<String, ResponseError> {
    log::debug!("fn: verify");

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
    pub username: String,
    pub role: String,
}

/// Returns API token
pub async fn login(host: String, login: LoginRequest) -> Result<LoginResponse, ResponseError> {
    log::debug!("fn: login");

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{host}/api/auth/login"))
        .body(serde_json::to_string(&login)?)
        .send()
        .await?
        .json::<LoginResponse>()
        .await?;

    log::info!(
        "logged in as {} ({})",
        response.user.username,
        response.user.role
    );

    Ok(response)
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct StatsResponse {
    pageviews: StatsResponseValue,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct StatsResponseValue {
    value: u64,
}

pub async fn pageviews_path(
    host: String,
    token: String,
    website_id: String,
    path: String,
) -> Result<u64, ResponseError> {
    log::debug!("fn: pageviews_path");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{host}/api/websites/{website_id}/stats"))
        .header("authorization", format!("Bearer {token}"))
        .query(&[("startAt", "0")])
        .query(&[("endAt", "9999999999999")])
        .query(&[("url", path)])
        .send()
        .await?
        .json::<StatsResponse>()
        .await?;

    Ok(response.pageviews.value)
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct MetricsResponseItem {
    x: String,
    y: u64,
}

pub type PageViewsPrefixResponse = HashMap<String, u64>;

pub async fn pageviews_prefix(
    host: String,
    token: String,
    website_id: String,
    prefix: String,
) -> Result<PageViewsPrefixResponse, ResponseError> {
    log::debug!("fn: pageviews_prefix");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{host}/api/websites/{website_id}/metrics"))
        .header("authorization", format!("Bearer {token}"))
        .query(&[("startAt", "0")])
        .query(&[("endAt", "9999999999999")])
        .query(&[("url", format!("~{prefix}/"))])
        .query(&[("type", "url")])
        .send()
        .await?
        .json::<Vec<MetricsResponseItem>>()
        .await?;

    let mut hashmap = PageViewsPrefixResponse::new();

    for item in response {
        hashmap.insert(item.x, item.y);
    }

    Ok(hashmap)
}
