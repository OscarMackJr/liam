use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct AnswerRequest<'a> {
    pub party_ref: &'a str,
    pub question: &'a str,
    pub purpose: &'a str,
    pub run_id: &'a str,
    pub nono_session_id: &'a str,
    pub step_role: &'static str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct AnswerResponse {
    pub trace_id: String,
    pub answer: String,
    pub context_complete: bool,
    pub proposal: Option<Proposal>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Proposal {
    pub action_type: String,
    pub party_ref: String,
    pub parameters: Value,
    pub justification: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum HometownError {
    NoInjectedKey,
    KeyInactive,
    PurposeRefused,
    Status(u16),
    Unavailable,
    InvalidResponse,
}
pub trait HometownClient {
    fn answer(&self, request: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError>;
}
pub struct HttpHometownClient {
    client: Client,
    endpoint: String,
}
impl HttpHometownClient {
    pub fn new(base_url: &str, proxy: &str, timeout_seconds: u64) -> Result<Self, HometownError> {
        let client = Client::builder()
            .proxy(reqwest::Proxy::all(proxy).map_err(|_| HometownError::Unavailable)?)
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .map_err(|_| HometownError::Unavailable)?;
        Ok(Self {
            client,
            endpoint: format!("{}/v1/agent/answer", base_url.trim_end_matches('/')),
        })
    }
}
impl HometownClient for HttpHometownClient {
    fn answer(&self, request: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(request)
            .send()
            .map_err(|_| HometownError::Unavailable)?;
        let status = response.status();
        if status.is_success() {
            return response.json().map_err(|_| HometownError::InvalidResponse);
        }
        if status.as_u16() == 401 {
            let inactive = response
                .text()
                .map(|body| body.contains("key_inactive"))
                .unwrap_or(false);
            return Err(if inactive {
                HometownError::KeyInactive
            } else {
                HometownError::NoInjectedKey
            });
        }
        if status.as_u16() == 403 {
            return Err(HometownError::PurposeRefused);
        }
        if status.is_server_error() {
            return Err(HometownError::Unavailable);
        }
        Err(HometownError::Status(status.as_u16()))
    }
}
