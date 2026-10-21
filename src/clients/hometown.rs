use super::reason;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
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
/// Extra top-level response keys are ignored (TC-SEC-03). The proposal is kept
/// as raw bytes so it can be forwarded to olive unchanged (D-01).
#[derive(Debug, Clone, Deserialize)]
pub struct AnswerResponse {
    pub trace_id: String,
    pub answer: String,
    pub context_complete: bool,
    #[serde(deserialize_with = "required_proposal")]
    pub proposal: Option<Box<RawValue>>,
}
/// S-HT-1 lists `proposal` as required and design §5.3 says it "is always
/// present, and null when there is none": JSON null is None, a missing key is
/// a parse error (InvalidResponse). Using deserialize_with without a default
/// stops serde from treating the absent Option as None.
fn required_proposal<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Box<RawValue>>, D::Error> {
    Option::<Box<RawValue>>::deserialize(deserializer)
}
/// Strings hold the full reason line text, e.g. `hometown:purpose_refused`.
#[derive(Debug, PartialEq, Eq)]
pub enum HometownError {
    PurposeRefused(String),
    Rejected { status: u16, reason: String },
    Unavailable,
    InvalidResponse,
}
impl HometownError {
    pub fn outcome(&self) -> &'static str {
        match self {
            HometownError::PurposeRefused(_) => "purpose_refused",
            HometownError::Rejected { reason, .. } => match reason.as_str() {
                "hometown:no_injected_key" => "hometown_no_injected_key",
                "hometown:key_inactive" => "hometown_key_inactive",
                _ => "hometown_failed",
            },
            HometownError::Unavailable | HometownError::InvalidResponse => "hometown_unavailable",
        }
    }
}
/// Status alone decides the class; the body only supplies a guarded reason.
pub(crate) fn classify(status: u16, body: &[u8]) -> HometownError {
    if status == 403 && super::error_code(body).as_deref() == Some("purpose_refused") {
        return HometownError::PurposeRefused(reason("hometown", status, body));
    }
    if status >= 500 {
        return HometownError::Unavailable;
    }
    HometownError::Rejected {
        status,
        reason: reason("hometown", status, body),
    }
}
pub trait HometownClient {
    fn answer(&self, request: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError>;
}
pub struct HttpHometownClient {
    client: Client,
    endpoint: String,
    timeout_seconds: u64,
}
impl HttpHometownClient {
    pub fn new(client: Client, base_url: &str, timeout_seconds: u64) -> Self {
        Self {
            client,
            endpoint: format!("{}/v1/agent/answer", base_url.trim_end_matches('/')),
            timeout_seconds,
        }
    }
}
impl HometownClient for HttpHometownClient {
    fn answer(&self, request: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError> {
        let response = self
            .client
            .post(&self.endpoint)
            .timeout(Duration::from_secs(self.timeout_seconds))
            .json(request)
            .send()
            .map_err(|_| HometownError::Unavailable)?;
        let status = response.status();
        let body = response.bytes().map_err(|_| HometownError::Unavailable)?;
        if status.is_success() {
            return serde_json::from_slice(&body).map_err(|_| HometownError::InvalidResponse);
        }
        Err(classify(status.as_u16(), &body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rej(status: u16, reason: &str) -> HometownError {
        HometownError::Rejected {
            status,
            reason: reason.into(),
        }
    }

    #[test]
    fn status_alone_decides_the_class() {
        assert_eq!(
            classify(403, br#"{"error":"purpose_refused"}"#),
            HometownError::PurposeRefused("hometown:purpose_refused".into())
        );
        assert_eq!(classify(403, b""), rej(403, "hometown:403"));
        assert_eq!(classify(403, b"not json"), rej(403, "hometown:403"));
        assert_eq!(
            classify(403, br#"{"error":"forbidden_thing"}"#),
            rej(403, "hometown:forbidden_thing")
        );
        assert_eq!(classify(401, b"{}"), rej(401, "hometown:401"));
        assert_eq!(
            classify(429, br#"{"error":"rate_limited"}"#),
            rej(429, "hometown:rate_limited")
        );
        assert_eq!(
            classify(500, br#"{"error":"x"}"#),
            HometownError::Unavailable
        );
        assert_eq!(classify(503, b""), HometownError::Unavailable);
        assert_eq!(
            classify(400, b"<html>secret text</html>"),
            rej(400, "hometown:400")
        );
    }

    #[test]
    fn outcomes_derive_from_guarded_codes() {
        assert_eq!(
            classify(401, br#"{"error":"no_injected_key"}"#).outcome(),
            "hometown_no_injected_key"
        );
        assert_eq!(
            classify(401, br#"{"error":"key_inactive"}"#).outcome(),
            "hometown_key_inactive"
        );
        assert_eq!(classify(404, b"").outcome(), "hometown_failed");
        assert_eq!(classify(403, b"").outcome(), "hometown_failed");
        assert_eq!(
            classify(403, br#"{"error":"purpose_refused"}"#).outcome(),
            "purpose_refused"
        );
        assert_eq!(
            HometownError::InvalidResponse.outcome(),
            "hometown_unavailable"
        );
    }

    #[test]
    fn proposal_key_is_required_but_may_be_null() {
        let missing = br#"{"trace_id":"t","answer":"a","context_complete":true}"#;
        assert!(serde_json::from_slice::<AnswerResponse>(missing).is_err());
        let null = br#"{"trace_id":"t","answer":"a","context_complete":true,"proposal":null}"#;
        assert!(serde_json::from_slice::<AnswerResponse>(null)
            .unwrap()
            .proposal
            .is_none());
    }

    #[test]
    fn response_ignores_extra_keys_and_keeps_raw_proposal() {
        let body = br#"{"trace_id":"t","answer":"a","context_complete":true,"x":1,"proposal":{"b":1.50,"a":"00e9"}}"#;
        let parsed: AnswerResponse = serde_json::from_slice(body).unwrap();
        assert_eq!(parsed.proposal.unwrap().get(), r#"{"b":1.50,"a":"00e9"}"#);
    }
}
