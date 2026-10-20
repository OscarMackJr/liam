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
    layers_outside_expectancy_scope: &'a [(String, crate::receipt::LayerStatus)],
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
            lines: vec!["{\"event\":\"run.started\"}".to_owned()],
            started_at,
        }
    }
    pub fn event(&mut self, event: &str) {
        self.lines.push(format!("{{\"event\":\"{event}\"}}"));
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
        proposal: Option<(&str, &str)>,
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
        let proposal = proposal.map(|(proposal_id, status)| ProposalRecord {
            proposal_id,
            status,
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
            layers_outside_expectancy_scope: vec![(
                "RestrictedToken".into(),
                LayerStatus::Unconfirmed,
            )],
        };
        evidence
            .finish(
                &session,
                "liam",
                "0.4.0",
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
    }
}
