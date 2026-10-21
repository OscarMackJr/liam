#![cfg(feature = "stubs")]
//! Evidence TCs (TC-EVD-01..05). Records are validated by hand against
//! `contracts/run-record.schema.json` (no JSON-schema crate is vendored).
//! Evidence-write failure (exit 30) is covered in `tests/process_contract.rs`.

mod common;

use common::{read_run_record, run_liam, write_test_config};
use liam::config::sha256_hex;
use liam::evidence::Evidence;
use liam::stubs::{Reply, ScriptedStub, StubRecordPair, TestProxy};
use serde_json::Value;
use std::{path::PathBuf, process::Output};

const EXE: &str = env!("CARGO_BIN_EXE_liam");
const CANARY: &str = "CANARY-7f3a";
const SCHEMA: &str = include_str!("../contracts/run-record.schema.json");

struct Case {
    output: Output,
    runs: PathBuf,
    _proxy: TestProxy,
    _fixture: StubRecordPair,
}

fn run_case(hometown: &ScriptedStub, olive: &ScriptedStub, extra_env: &[(&str, &str)]) -> Case {
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    write_test_config(
        fixture.directory(),
        hometown.base_url(),
        olive.base_url(),
        2,
    );
    let output = run_liam(
        EXE,
        &fixture,
        proxy.url(),
        &[
            "--purpose",
            "servicing",
            "--party",
            CANARY,
            "--question",
            CANARY,
        ],
        extra_env,
    );
    Case {
        output,
        runs,
        _proxy: proxy,
        _fixture: fixture,
    }
}

fn proposal_answer() -> String {
    serde_json::json!({
        "trace_id": "trace-stub-1",
        "answer": CANARY,
        "context_complete": true,
        "proposal": {
            "action_type": "crm.case_note@1",
            "party_ref": CANARY,
            "parameters": {"note": CANARY},
            "justification": CANARY
        }
    })
    .to_string()
}

fn matches_type(value: &Value, ty: &str) -> bool {
    match ty {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => false,
    }
}

fn is_hex64(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_code(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 64
        && text.as_bytes()[0].is_ascii_lowercase()
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Returns the first violation, or None. Handles the subset of JSON Schema the
/// run-record schema uses: type, pattern (two known), required, properties,
/// additionalProperties:false, items, oneOf.
fn violation(schema: &Value, value: &Value, path: &str) -> Option<String> {
    if let Some(options) = schema.get("oneOf").and_then(Value::as_array) {
        if options.iter().any(|o| violation(o, value, path).is_none()) {
            return None;
        }
        return Some(format!("{path}: matches no oneOf branch: {value}"));
    }
    if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        if !matches_type(value, ty) {
            return Some(format!("{path}: expected {ty}, got {value}"));
        }
    }
    if let (Some(text), Some(pattern)) = (
        value.as_str(),
        schema.get("pattern").and_then(Value::as_str),
    ) {
        let ok = match pattern {
            "^[0-9a-f]{64}$" => is_hex64(text),
            "^[a-z][a-z0-9_]{0,63}$" => is_code(text),
            other => return Some(format!("{path}: unsupported pattern {other}")),
        };
        if !ok {
            return Some(format!("{path}: {text:?} violates {pattern}"));
        }
    }
    if let Some(object) = value.as_object() {
        for key in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let key = key.as_str().unwrap();
            if !object.contains_key(key) {
                return Some(format!("{path}: missing required {key}"));
            }
        }
        let props = schema.get("properties").and_then(Value::as_object);
        for (key, child) in object {
            match props.and_then(|p| p.get(key)) {
                Some(child_schema) => {
                    if let Some(v) = violation(child_schema, child, &format!("{path}.{key}")) {
                        return Some(v);
                    }
                }
                None if schema.get("additionalProperties") == Some(&Value::Bool(false)) => {
                    return Some(format!("{path}: key {key} not in schema"));
                }
                None => {}
            }
        }
    }
    if let (Some(items), Some(array)) = (schema.get("items"), value.as_array()) {
        for (i, item) in array.iter().enumerate() {
            if let Some(v) = violation(items, item, &format!("{path}[{i}]")) {
                return Some(v);
            }
        }
    }
    None
}

fn schema() -> Value {
    serde_json::from_str(SCHEMA).expect("run-record.schema.json parses")
}

fn validate_record(record: &Value) {
    if let Some(v) = violation(&schema(), record, "$") {
        panic!("{v}");
    }
}

fn assert_no_canary(case: &Case, allow_stdout: bool) {
    let (record, log) = read_run_record(&case.runs);
    let text = format!("{record}{}", String::from_utf8_lossy(&log));
    assert!(!text.contains(CANARY), "canary in record or log");
    assert!(!String::from_utf8_lossy(&case.output.stderr).contains(CANARY));
    if !allow_stdout {
        assert!(!String::from_utf8_lossy(&case.output.stdout).contains(CANARY));
    }
}

fn completed_case() -> Case {
    let hometown = ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]);
    let olive = ScriptedStub::olive_ok(202, "prop-1", "awaiting_approval", None);
    let case = run_case(&hometown, &olive, &[]);
    assert_eq!(
        case.output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&case.output.stderr)
    );
    case
}

fn last_line(log: &[u8]) -> String {
    String::from_utf8_lossy(log)
        .lines()
        .last()
        .unwrap()
        .to_owned()
}

#[test]
fn tc_evd_01_completed_record_matches_schema() {
    let case = completed_case();
    let (record, log) = read_run_record(&case.runs);
    validate_record(&record);
    assert_eq!(record["outcome"], "completed");
    assert!(record["proposal"]["reason"].is_null(), "D-07: null reason");
    assert_eq!(record["proposal"]["proposal_id"], "prop-1");
    assert_eq!(record["answer"]["trace_id"], "trace-stub-1");
    assert_eq!(record["log_sha256"], sha256_hex(&log));
    assert!(!record["nono"]["session_id"].as_str().unwrap().is_empty());
    assert!(!record["run_id"].as_str().unwrap().is_empty());
    for layer in record["nono"]["layers_outside_expectancy_scope"]
        .as_array()
        .unwrap()
    {
        assert!(layer.is_object(), "spec 8.1: {{id, status}} objects");
        assert!(layer["id"].is_string() && layer["status"].is_string());
    }
    assert!(last_line(&log).contains("run.completed"));
}

#[test]
fn tc_evd_01_refused_policy_reason_is_string() {
    let hometown = ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]);
    let olive = ScriptedStub::olive_ok(200, "prop-2", "refused_policy", Some("policy_denied"));
    let case = run_case(&hometown, &olive, &[]);
    assert_eq!(case.output.status.code(), Some(0));
    let (record, _) = read_run_record(&case.runs);
    validate_record(&record);
    assert_eq!(record["proposal"]["reason"], "policy_denied");
}

#[test]
fn tc_evd_01_schema_check_rejects_extra_and_missing_keys() {
    let case = completed_case();
    let (mut record, _) = read_run_record(&case.runs);
    record["leaked"] = Value::from("x");
    assert!(violation(&schema(), &record, "$").is_some());
    let (mut record, _) = read_run_record(&case.runs);
    record.as_object_mut().unwrap().remove("log_sha256");
    assert!(violation(&schema(), &record, "$").is_some());
}

#[test]
fn tc_evd_02_canary_only_in_stdout() {
    let case = completed_case();
    assert_no_canary(&case, true);
    let stdout = String::from_utf8_lossy(&case.output.stdout);
    assert!(stdout.contains(CANARY), "answer text is shown on stdout");
}

#[test]
fn tc_evd_03_party_ref_hash_only() {
    let case = completed_case();
    let (record, _) = read_run_record(&case.runs);
    assert_eq!(record["party_ref_hash"], sha256_hex(CANARY.as_bytes()));
    assert!(record.get("party_ref").is_none());
    assert!(record["proposal"].get("party_ref").is_none());
}

#[test]
fn tc_evd_04_hometown_5xx_twice_records_failure() {
    let hometown = ScriptedStub::new(vec![Reply::empty(500), Reply::empty(500)]);
    let olive = ScriptedStub::new(vec![Reply::empty(500)]);
    let case = run_case(&hometown, &olive, &[]);
    assert_eq!(case.output.status.code(), Some(21));
    assert_eq!(hometown.count(), 2);
    let (record, log) = read_run_record(&case.runs);
    validate_record(&record);
    assert_eq!(record["outcome"], "hometown_unavailable");
    assert!(record["proposal"].is_null());
    assert_eq!(record["log_sha256"], sha256_hex(&log));
    assert!(last_line(&log).contains("run.failed"));
    assert_no_canary(&case, false);
}

#[test]
fn tc_evd_04_olive_failure_keeps_answer_ids() {
    let hometown = ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]);
    let olive = ScriptedStub::error(500, "unavailable");
    let case = run_case(&hometown, &olive, &[]);
    assert_eq!(case.output.status.code(), Some(22));
    let (record, _) = read_run_record(&case.runs);
    validate_record(&record);
    assert_eq!(record["answer"]["trace_id"], "trace-stub-1");
    assert_eq!(record["answer"]["context_complete"], true);
    assert!(record["proposal"].is_null());
    assert_no_canary(&case, true);
}

#[test]
fn tc_evd_05_self_check_failure_writes_nothing() {
    let hometown = ScriptedStub::new(vec![Reply::empty(500)]);
    let olive = ScriptedStub::new(vec![Reply::empty(500)]);
    let case = run_case(
        &hometown,
        &olive,
        &[("NONO_RECEIPT_PATH", "C:/nonexistent-liam-receipt.json")],
    );
    assert_eq!(case.output.status.code(), Some(10));
    assert_eq!(hometown.count(), 0);
    let files = std::fs::read_dir(&case.runs)
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(files, 0, "no run files after a startup failure");
}

#[test]
fn hostile_event_field_stays_one_json_line() {
    let dir = std::env::temp_dir().join(format!("liam-record-{}", uuid::Uuid::now_v7()));
    let mut evidence = Evidence::new(dir.to_string_lossy().into_owned(), "hostile".into(), 1);
    evidence.event_with(
        "proposal.suppressed",
        serde_json::json!({"reason": "a\"b\nc\r\n{\"event\":\"forged\"}\\"}),
    );
    let session = liam::receipt::ValidatedSession {
        session_id: "s".into(),
        agent_key_id: "k".into(),
        token_arm: "A".into(),
        layers_outside_expectancy_scope: vec![],
    };
    evidence
        .finish(
            &session,
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
    let log = std::fs::read_to_string(dir.join("hostile.jsonl")).unwrap();
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines.len(), 3, "run.started, one event, run.completed");
    for line in &lines {
        let v: Value = serde_json::from_str(line).unwrap();
        assert_ne!(v["event"], "forged");
    }
}

#[test]
fn layer_entries_are_id_status_objects() {
    let dir = std::env::temp_dir().join(format!("liam-record-{}", uuid::Uuid::now_v7()));
    let mut evidence = Evidence::new(dir.to_string_lossy().into_owned(), "layers".into(), 1);
    let session = liam::receipt::ValidatedSession {
        session_id: "s".into(),
        agent_key_id: "k".into(),
        token_arm: "A".into(),
        layers_outside_expectancy_scope: vec![(
            "RestrictedToken".into(),
            liam::receipt::LayerStatus::Unconfirmed,
        )],
    };
    evidence
        .finish(
            &session,
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
    let record: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("layers.json")).unwrap()).unwrap();
    validate_record(&record);
    assert_eq!(
        record["nono"]["layers_outside_expectancy_scope"],
        serde_json::json!([{"id": "RestrictedToken", "status": "Unconfirmed"}])
    );
}
