pub mod bus_factor;
pub mod repository;

use crate::BusFactorQueryCommand;
use anyhow::{anyhow, Result};
use reqwest::header::HeaderMap;
use std::env;
use std::sync::Arc;

const USER_AGENT_VALUE: &str = "request";

#[derive(Debug)]
pub struct HttpClientDetails {
    client: reqwest::Client,
    command: BusFactorQueryCommand,
}
#[derive(Debug)]
pub struct BusFactorQueryResult {
    pub login: String,
    pub contributions: u32,
    pub repo_name: String,
    pub bus_factor: u32,
    pub stargazers: u32,
}
#[derive(Debug, Clone)]
pub struct RepositoryQueryResult {
    pub(crate) stargazers: u32,
    pub(crate) contributor_url: String,
    pub(crate) project_name: String,
    pub(crate) client_details: Arc<HttpClientDetails>,
}

impl HttpClientDetails {
    fn new(command: BusFactorQueryCommand) -> Result<Self> {
        let personal_access_token = env::var("GITHUB_ACCESS_TOKEN")
            .map(|token| format!("token {}", token))
            .map_err(|err| anyhow!("Missing GITHUB_ACCESS_TOKEN environment variable. {err}"))?;

        let mut default_headers = HeaderMap::new();
        default_headers.append(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&personal_access_token)?,
        );
        default_headers.append(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_str(USER_AGENT_VALUE)?,
        );
        Ok(HttpClientDetails {
            client: reqwest::ClientBuilder::default()
                .default_headers(default_headers)
                .build()?,
            command,
        })
    }
}
pub type Other = serde_json::Map<String, serde_json::Value>;
