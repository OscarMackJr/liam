use super::hometown::Proposal;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[derive(Debug, Clone, Serialize)]
pub struct ProposalRequest<'a> {
    #[serde(flatten)]
    pub proposal: &'a Proposal,
    pub run_id: &'a str,
    pub trace_ref: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ProposalResponse {
    pub proposal_id: String,
    pub status: String,
    #[serde(default)]
    pub reason: Option<String>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum OliveError {
    Unavailable,
    Status(u16),
    InvalidResponse,
}
pub trait OliveClient {
    fn submit(&self, request: &ProposalRequest<'_>) -> Result<ProposalResponse, OliveError>;
}
pub struct HttpOliveClient {
    client: Client,
    endpoint: String,
}
impl HttpOliveClient {
    pub fn new(base_url: &str, proxy: &str, timeout_seconds: u64) -> Result<Self, OliveError> {
        let client = Client::builder()
            .proxy(reqwest::Proxy::all(proxy).map_err(|_| OliveError::Unavailable)?)
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|_| OliveError::Unavailable)?;
        Ok(Self {
            client,
            endpoint: format!("{}/v1/proposals", base_url.trim_end_matches('/')),
        })
    }
}
impl OliveClient for HttpOliveClient {
    fn submit(&self, request: &ProposalRequest<'_>) -> Result<ProposalResponse, OliveError> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(request)
            .send()
            .map_err(|_| OliveError::Unavailable)?;
        if !response.status().is_success() {
            return Err(OliveError::Status(response.status().as_u16()));
        }
        response.json().map_err(|_| OliveError::InvalidResponse)
    }
}
