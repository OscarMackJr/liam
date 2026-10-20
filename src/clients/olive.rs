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
    timeout_seconds: u64,
}
impl HttpOliveClient {
    pub fn new(client: Client, base_url: &str, timeout_seconds: u64) -> Self {
        Self {
            client,
            endpoint: format!("{}/v1/proposals", base_url.trim_end_matches('/')),
            timeout_seconds,
        }
    }
}
impl OliveClient for HttpOliveClient {
    fn submit(&self, request: &ProposalRequest<'_>) -> Result<ProposalResponse, OliveError> {
        let response = self
            .client
            .post(&self.endpoint)
            .timeout(Duration::from_secs(self.timeout_seconds))
            .json(request)
            .send()
            .map_err(|_| OliveError::Unavailable)?;
        if !response.status().is_success() {
            return Err(OliveError::Status(response.status().as_u16()));
        }
        response.json().map_err(|_| OliveError::InvalidResponse)
    }
}
