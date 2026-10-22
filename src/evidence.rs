//! Content-free local evidence.  It deliberately serializes only join keys,
//! status and hashes; callers never pass prompt or response text to this API.

use crate::config::sha256_hex;
use crate::receipt::ValidatedSession;
use serde::Serialize;
use std::{fs, io, path::Path};

#[derive(Serialize)]
struct RunRecord<'a> {
    run_id: &'a str,
    agent_id: &'a str,
    agent_version: &'a str,
    started_at: u64,
    ended_at: u64,
    outcome: &'a str,
    nono: NonoRecord<'a>,
    purpose: &'a str,
    party_ref_hash: &'a str,
    // Failed-olive runs also carry the answer ids (assumption A7; additive, content-free).
    answer: Option<AnswerRecord<'a>>,
    proposal: Option<ProposalRecord<'a>>,
    log_sha256: &'a str,
}
#[derive(Serialize)]
struct NonoRecord<'a> {
    session_id: &'a str,
    profile_version: u32,
    token_arm: &'a str,
    receipt_integrity: &'static str,
    agent_key_id: &'a str,
    #[serde(serialize_with = "layer_objects")]
    layers_outside_expectancy_scope: &'a [(String, crate::receipt::LayerStatus)],
}
fn layer_objects<S: serde::Serializer>(
    layers: &&[(String, crate::receipt::LayerStatus)],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    #[derive(Serialize)]
    struct Layer<'a> {
        id: &'a str,
        status: crate::receipt::LayerStatus,
    }
    let mut seq = serializer.serialize_seq(Some(layers.len()))?;
    for (id, status) in layers.iter() {
        seq.serialize_element(&Layer {
            id,
            status: *status,
        })?;
    }
    seq.end()
}
#[derive(Serialize)]
struct AnswerRecord<'a> {
    trace_id: &'a str,
    context_complete: bool,
}
#[derive(Serialize)]
struct ProposalRecord<'a> {
    proposal_id: &'a str,
    status: &'a str,
    reason: Option<&'a str>,
}

/// Proposal outcome for the run record (D-07). `reason` is only set for
/// refused_policy and must be a guarded code.
pub struct ProposalOutcome<'a> {
    pub proposal_id: &'a str,
    pub status: &'a str,
    pub reason: Option<&'a str>,
}

pub struct Evidence {
    dir: String,
    run_id: String,
    lines: Vec<String>,
    started_at: u64,
}
impl Evidence {
    pub fn new(dir: String, run_id: String, started_at: u64) -> Self {
        Self {
            dir,
            run_id,
            lines: vec![serde_json::json!({"event": "run.started"}).to_string()],
            started_at,
        }
    }
    pub fn event(&mut self, event: &str) {
        self.event_with(event, serde_json::Value::Null);
    }
    /// Serde-built event line: `{"event": name, ...fields}` (D-08, D-09).
    pub fn event_with(&mut self, name: &str, fields: serde_json::Value) {
        let mut line = serde_json::Map::new();
        line.insert("event".into(), serde_json::Value::String(name.to_owned()));
        if let serde_json::Value::Object(map) = fields {
            for (key, value) in map {
                line.insert(key, value);
            }
        }
        self.lines.push(serde_json::Value::Object(line).to_string());
    }
    #[allow(clippy::too_many_arguments)]
    pub fn finish(
        &mut self,
        session: &ValidatedSession,
        agent_id: &str,
        version: &str,
        profile_version: u32,
        purpose: &str,
        party_ref: &str,
        outcome: &str,
        answer: Option<(&str, bool)>,
        proposal: Option<ProposalOutcome<'_>>,
        ended_at: u64,
    ) -> io::Result<()> {
        self.event(if outcome == "completed" {
            "run.completed"
        } else {
            "run.failed"
        });
        fs::create_dir_all(&self.dir)?;
        let log = format!("{}\n", self.lines.join("\n"));
        let log_hash = sha256_hex(log.as_bytes());
        let log_path = Path::new(&self.dir).join(format!("{}.jsonl", self.run_id));
        fs::write(log_path, log)?;
        let answer = answer.map(|(trace_id, context_complete)| AnswerRecord {
            trace_id,
            context_complete,
        });
        let proposal = proposal.map(|p| ProposalRecord {
            proposal_id: p.proposal_id,
            status: p.status,
            reason: p.reason.map(|r| {
                if crate::clients::guard_code(r) {
                    r
                } else {
                    "invalid_reason"
                }
            }),
        });
        let record = RunRecord {
            run_id: &self.run_id,
            agent_id,
            agent_version: version,
            started_at: self.started_at,
            ended_at,
            outcome,
            nono: NonoRecord {
                session_id: &session.session_id,
                profile_version,
                token_arm: &session.token_arm,
                receipt_integrity: "acl-only",
                agent_key_id: &session.agent_key_id,
                layers_outside_expectancy_scope: &session.layers_outside_expectancy_scope,
            },
            purpose,
            party_ref_hash: &sha256_hex(party_ref.as_bytes()),
            answer,
            proposal,
            log_sha256: &log_hash,
        };
        let record_json = serde_json::to_vec_pretty(&record).map_err(io::Error::other)?;
        fs::write(
            Path::new(&self.dir).join(format!("{}.json", self.run_id)),
            record_json,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receipt::{LayerStatus, ValidatedSession};
    #[test]
    fn keeps_content_out_of_local_evidence() {
        let dir = std::env::temp_dir().join(format!("liam-evidence-{}", uuid::Uuid::now_v7()));
        let mut evidence = Evidence::new(dir.to_string_lossy().into_owned(), "run-1".into(), 1);
        let session = ValidatedSession {
            session_id: "session-1".into(),
            agent_key_id: "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b".into(),
            token_arm: "StubArm".into(),
            proxy_port: 50413,
            layers_outside_expectancy_scope: vec![(
                "RestrictedToken".into(),
                LayerStatus::Unconfirmed,
            )],
        };
        evidence
            .finish(
                &session,
                "liam",
                "0.4.1",
                4,
                "servicing",
                "CANARY-7f3a",
                "completed",
                Some(("trace-1", true)),
                None,
                2,
            )
            .unwrap();
        let record = std::fs::read_to_string(dir.join("run-1.json")).unwrap();
        let log = std::fs::read_to_string(dir.join("run-1.jsonl")).unwrap();
        assert!(!record.contains("CANARY-7f3a") && !log.contains("CANARY-7f3a"));
        let parsed: serde_json::Value = serde_json::from_str(&record).unwrap();
        assert_eq!(
            parsed["nono"]["layers_outside_expectancy_scope"],
            serde_json::json!([{"id": "RestrictedToken", "status": "Unconfirmed"}])
        );
    }

    fn session() -> ValidatedSession {
        ValidatedSession {
            session_id: "s".into(),
            agent_key_id: "k".into(),
            token_arm: "A".into(),
            proxy_port: 50413,
            layers_outside_expectancy_scope: vec![],
        }
    }

    #[test]
    fn events_are_valid_json_even_with_hostile_fields() {
        let dir = std::env::temp_dir().join(format!("liam-evidence-{}", uuid::Uuid::now_v7()));
        let mut evidence = Evidence::new(dir.to_string_lossy().into_owned(), "run-2".into(), 1);
        evidence.event_with(
            "proposal.refused",
            serde_json::json!({"reason": "a\"b\nc\\d", "investigate": true}),
        );
        evidence
            .finish(
                &session(),
                "liam",
                "0.4.1",
                4,
                "p",
                "x",
                "completed",
                None,
                None,
                2,
            )
            .unwrap();
        let log = std::fs::read_to_string(dir.join("run-2.jsonl")).unwrap();
        for line in log.lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(v.get("event").is_some());
        }
        assert!(log.contains("\"investigate\":true"));
    }

    #[test]
    fn proposal_record_carries_reason_or_null() {
        let dir = std::env::temp_dir().join(format!("liam-evidence-{}", uuid::Uuid::now_v7()));
        for (id, reason, expect) in [
            ("r3", None, "null"),
            (
                "r4",
                Some("trace_proposal_mismatch"),
                "\"trace_proposal_mismatch\"",
            ),
        ] {
            let mut evidence = Evidence::new(dir.to_string_lossy().into_owned(), id.into(), 1);
            evidence
                .finish(
                    &session(),
                    "liam",
                    "0.4.1",
                    4,
                    "p",
                    "x",
                    "completed",
                    None,
                    Some(ProposalOutcome {
                        proposal_id: "pr",
                        status: "s",
                        reason,
                    }),
                    2,
                )
                .unwrap();
            let v: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(dir.join(format!("{id}.json"))).unwrap(),
            )
            .unwrap();
            assert_eq!(v["proposal"]["reason"].to_string(), expect);
        }
    }
}
