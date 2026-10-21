use super::{guard_code, reason};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::time::Duration;

/// The four proposal fields are borrowed raw bytes from hometown's response, so
/// the forwarded body is byte-identical (D-01). Never merge a RawValue via serde attributes.
#[derive(Debug, Serialize)]
pub struct ProposalRequest<'a> {
    pub action_type: &'a RawValue,
    pub party_ref: &'a RawValue,
    pub parameters: &'a RawValue,
    pub justification: &'a RawValue,
    pub run_id: &'a str,
    pub trace_ref: &'a str,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ProposalResponse {
    pub proposal_id: String,
    pub status: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(skip)]
    pub http_status: u16,
}
#[derive(Debug, PartialEq, Eq)]
pub enum OliveError {
    Rejected { status: u16, reason: String },
    Unavailable,
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
/// The vocabulary's `action_outcomes` (design §7.2: "Status values are the
/// vocabulary's action_outcomes"). Kept in step with the SHA-pinned
/// `contracts/vendored/vocabulary.yaml` by a unit test.
pub const ACTION_OUTCOMES: &[&str] = &[
    "proposed",
    "refused_policy",
    "awaiting_approval",
    "approved",
    "rejected",
    "executed",
    "execution_failed",
    "expired",
];

/// A 202 is always `awaiting_approval`; a 200 is `refused_policy` or an
/// idempotent replay recorded as the returned status (§7.4, TC-ACT-09). Any
/// status outside the vocabulary is not a proposal response (S-OL-1:
/// olive:unavailable), so free text never reaches the record or stdout.
fn status_ok(http_status: u16, status: &str) -> bool {
    ACTION_OUTCOMES.contains(&status) && (http_status != 202 || status == "awaiting_approval")
}

pub(crate) fn classify(status: u16, body: &[u8]) -> Result<ProposalResponse, OliveError> {
    if status == 200 || status == 202 {
        let mut parsed: ProposalResponse =
            serde_json::from_slice(body).map_err(|_| OliveError::InvalidResponse)?;
        // Liam-side id guard (pending confirmation in S-OL-1): an empty or
        // free-text proposal_id never reaches the record, the log or stdout.
        if !status_ok(status, &parsed.status) || !super::id_ok(&parsed.proposal_id) {
            return Err(OliveError::InvalidResponse);
        }
        parsed.http_status = status;
        if !parsed.reason.as_deref().is_some_and(guard_code) {
            parsed.reason = None;
        }
        // S-OL-1/§7.2: refused_policy carries a reason code. Without a guarded
        // one it is not a valid proposal response, so the log event and the
        // run record never disagree about the reason.
        if parsed.status == "refused_policy" && parsed.reason.is_none() {
            return Err(OliveError::InvalidResponse);
        }
        return Ok(parsed);
    }
    if status >= 500 || !(400..500).contains(&status) {
        return Err(OliveError::Unavailable);
    }
    Err(OliveError::Rejected {
        status,
        reason: reason("olive", status, body),
    })
}
impl OliveClient for HttpOliveClient {
    fn submit(&self, request: &ProposalRequest<'_>) -> Result<ProposalResponse, OliveError> {
        let payload = serde_json::to_vec(request).map_err(|_| OliveError::InvalidResponse)?;
        let response = self
            .client
            .post(&self.endpoint)
            .timeout(Duration::from_secs(self.timeout_seconds))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .map_err(|_| OliveError::Unavailable)?;
        let status = response.status().as_u16();
        let body = response.bytes().map_err(|_| OliveError::Unavailable)?;
        classify(status, &body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(s: &str) -> Box<RawValue> {
        RawValue::from_string(s.to_string()).unwrap()
    }

    #[test]
    fn request_embeds_raw_bytes_verbatim() {
        let (a, p, m, j) = (
            raw(r#""crm.case_note@1""#),
            raw(r#""opaque""#),
            raw(r#"{ "z":1.50,  "a":"00e9", "n": "é" }"#),
            raw(r#""why  00e9""#),
        );
        let body = serde_json::to_string(&ProposalRequest {
            action_type: &a,
            party_ref: &p,
            parameters: &m,
            justification: &j,
            run_id: "r",
            trace_ref: "t",
        })
        .unwrap();
        for field in [a.get(), p.get(), m.get(), j.get()] {
            assert!(body.contains(field), "{field} missing from {body}");
        }
    }

    #[test]
    fn accepts_200_and_202_and_guards_reason() {
        let ok = br#"{"proposal_id":"p","status":"awaiting_approval","reason":"Bad Text"}"#;
        for s in [200, 202] {
            let r = classify(s, ok).unwrap();
            assert_eq!(r.http_status, s);
            assert!(r.reason.is_none());
        }
        let good = br#"{"proposal_id":"p","status":"refused_policy","reason":"policy"}"#;
        assert_eq!(
            classify(200, good).unwrap().reason.as_deref(),
            Some("policy")
        );
    }

    #[test]
    fn failures_classify_by_status() {
        assert_eq!(
            classify(400, br#"{"error":"bad_thing"}"#).unwrap_err(),
            OliveError::Rejected {
                status: 400,
                reason: "olive:bad_thing".into()
            }
        );
        assert_eq!(
            classify(404, b"x").unwrap_err(),
            OliveError::Rejected {
                status: 404,
                reason: "olive:404".into()
            }
        );
        assert_eq!(classify(500, b"{}").unwrap_err(), OliveError::Unavailable);
        assert_eq!(
            classify(200, b"nope").unwrap_err(),
            OliveError::InvalidResponse
        );
    }

    #[test]
    fn refused_policy_requires_a_guarded_reason() {
        for body in [
            &br#"{"proposal_id":"p","status":"refused_policy"}"#[..],
            br#"{"proposal_id":"p","status":"refused_policy","reason":null}"#,
            br#"{"proposal_id":"p","status":"refused_policy","reason":"Bad Text"}"#,
            br#"{"proposal_id":"p","status":"refused_policy","reason":""}"#,
        ] {
            assert_eq!(
                classify(200, body).unwrap_err(),
                OliveError::InvalidResponse
            );
        }
    }

    #[test]
    fn status_must_be_a_vocabulary_outcome() {
        let body = |s: &str| format!(r#"{{"proposal_id":"p","status":"{s}","reason":"r"}}"#);
        for bad in [
            "",
            "Approved",
            "awaiting approval",
            "CANARY-7f3a",
            "ok\\u001b[2K",
        ] {
            for http in [200, 202] {
                assert_eq!(
                    classify(http, body(bad).as_bytes()).unwrap_err(),
                    OliveError::InvalidResponse,
                    "{http} {bad}"
                );
            }
        }
        // 202 is only ever awaiting_approval
        assert_eq!(
            classify(202, body("executed").as_bytes()).unwrap_err(),
            OliveError::InvalidResponse
        );
        // 200 replay is recorded as the returned status (TC-ACT-09)
        for ok in ACTION_OUTCOMES {
            assert_eq!(classify(200, body(ok).as_bytes()).unwrap().status, *ok);
        }
        assert!(classify(202, body("awaiting_approval").as_bytes()).is_ok());
    }

    #[test]
    fn proposal_id_must_pass_the_id_guard() {
        let body = |id: &str| {
            format!(
                r#"{{"proposal_id":{},"status":"awaiting_approval"}}"#,
                serde_json::to_string(id).unwrap()
            )
        };
        assert!(classify(202, body("proposal-1").as_bytes()).is_ok());
        for bad in ["", "has space", "p/q", "p\u{1b}[2J", &"a".repeat(129)] {
            for http in [200, 202] {
                assert_eq!(
                    classify(http, body(bad).as_bytes()).unwrap_err(),
                    OliveError::InvalidResponse,
                    "{http} {bad:?}"
                );
            }
        }
    }

    #[test]
    fn action_outcomes_match_the_vendored_vocabulary() {
        let yaml = include_str!("../../contracts/vendored/vocabulary.yaml");
        let mut listed = Vec::new();
        let mut inside = false;
        for line in yaml.lines() {
            if line.trim_end() == "action_outcomes:" {
                inside = true;
                continue;
            }
            if inside {
                match line.trim().strip_prefix("- ") {
                    Some(item) if line.starts_with(char::is_whitespace) => listed.push(item),
                    _ => break,
                }
            }
        }
        assert_eq!(listed, ACTION_OUTCOMES);
    }
}
