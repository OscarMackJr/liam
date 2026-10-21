#![cfg(feature = "stubs")]

mod common;

use common::{read_run_record, run_liam, write_test_config};
use liam::stubs::{Reply, ScriptedStub, StubRecordPair, TestProxy};
use std::{path::PathBuf, process::Output, time::Duration};

const EXE: &str = env!("CARGO_BIN_EXE_liam");
const CANARY: &str = "CANARY-7f3a";
const ANSWER: &str =
    r#"{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":null}"#;

fn proposal_answer() -> String {
    serde_json::json!({
        "trace_id": "trace-stub-1",
        "answer": "stub answer",
        "context_complete": true,
        "proposal": {
            "action_type": "crm.case_note@1",
            "party_ref": CANARY,
            "parameters": {"note": "stub"},
            "justification": "stub justification"
        }
    })
    .to_string()
}

fn error_body(code: &str) -> String {
    serde_json::json!({ "error": code }).to_string()
}

struct Case {
    output: Output,
    proxy: TestProxy,
    runs: PathBuf,
    _fixture: StubRecordPair,
}

fn run_case(hometown: &ScriptedStub, olive: &ScriptedStub, question: &str, timeout: u64) -> Case {
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    write_test_config(
        fixture.directory(),
        hometown.base_url(),
        olive.base_url(),
        timeout,
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
            question,
        ],
        &[],
    );
    Case {
        output,
        proxy,
        runs,
        _fixture: fixture,
    }
}

fn idle() -> ScriptedStub {
    ScriptedStub::new(vec![Reply::empty(500)])
}

impl Case {
    fn code(&self) -> i32 {
        self.output.status.code().unwrap_or(-1)
    }
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).into_owned()
    }
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }
    /// Exactly one stderr line `liam: <code> <reason>` (TC-FAIL-07).
    fn assert_failed(&self, code: i32, reason: &str) {
        assert_eq!(self.code(), code, "stderr: {}", self.stderr());
        assert_eq!(self.stderr(), format!("liam: {code} {reason}\n"));
        assert!(self.proxy.errors().is_empty(), "{:?}", self.proxy.errors());
    }
    /// Run record, log and stderr carry neither the canary nor `extra`.
    fn assert_content_free(&self, extra: &[&str]) {
        let (record, log) = read_run_record(&self.runs);
        let text = format!("{record}{}{}", String::from_utf8_lossy(&log), self.stderr());
        assert!(!text.contains(CANARY));
        for needle in extra {
            assert!(!text.contains(needle), "leaked {needle}");
        }
    }
    fn outcome(&self) -> String {
        let (record, _) = read_run_record(&self.runs);
        record["outcome"].as_str().unwrap_or_default().to_owned()
    }
}

// ---- TC-ANS ----

#[test]
fn tc_ans_01_answer_without_proposal() {
    let hometown = ScriptedStub::new(vec![Reply::json(200, ANSWER)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, CANARY, 2);
    assert_eq!(case.code(), 0, "stderr: {}", case.stderr());
    assert_eq!(case.stdout(), "stub answer\n");
    assert_eq!(case.proxy.stats().hometown_hits, 1);
    assert_eq!(case.proxy.stats().olive_hits, 0);
    let (record, log) = read_run_record(&case.runs);
    assert!(record["proposal"].is_null(), "{record}");
    assert_eq!(record["answer"]["trace_id"], "trace-stub-1");
    assert!(!record.to_string().contains(CANARY));
    assert!(!String::from_utf8_lossy(&log).contains(CANARY));
}

#[test]
fn tc_ans_02_incomplete_context_notice() {
    let body = r#"{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":false}"#;
    let hometown = ScriptedStub::new(vec![Reply::json(200, body)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    assert_eq!(case.code(), 0, "stderr: {}", case.stderr());
    assert!(
        case.stdout()
            .starts_with("Notice: answer may be incomplete.\n"),
        "{}",
        case.stdout()
    );
    let (record, _) = read_run_record(&case.runs);
    assert_eq!(record["answer"]["context_complete"], false);
}

#[test]
fn tc_ans_03_purpose_refused() {
    let hometown = ScriptedStub::error(403, "purpose_refused");
    let olive = idle();
    let case = run_case(&hometown, &olive, CANARY, 2);
    case.assert_failed(20, "hometown:purpose_refused");
    assert_eq!(hometown.count(), 1);
    assert_eq!(case.proxy.stats().olive_hits, 0);
    assert_eq!(case.outcome(), "purpose_refused");
    case.assert_content_free(&[]);
}

#[test]
fn tc_ans_03_other_403_is_not_purpose_refusal() {
    let hometown = ScriptedStub::new(vec![Reply::empty(403)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    case.assert_failed(21, "hometown:403");
    assert_eq!(hometown.count(), 1);
    assert_eq!(case.outcome(), "hometown_failed");
}

#[test]
fn tc_ans_03_other_coded_403_exits_21() {
    let hometown = ScriptedStub::error(403, "forbidden_thing");
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    case.assert_failed(21, "hometown:forbidden_thing");
    assert_eq!(hometown.count(), 1);
}

#[test]
fn tc_ans_05_coded_rejections_give_distinct_reasons() {
    for (status, code) in [
        (422, "agent_not_registered"),
        (422, "party_not_found"),
        (400, "invalid_request"),
        (409, "run_conflict"),
    ] {
        let hometown = ScriptedStub::error(status, code);
        let olive = idle();
        let case = run_case(&hometown, &olive, CANARY, 2);
        case.assert_failed(21, &format!("hometown:{code}"));
        assert_eq!(hometown.count(), 1, "{code}");
        assert_eq!(case.proxy.stats().olive_hits, 0);
        case.assert_content_free(&[]);
    }
}

#[test]
fn tc_ans_05_key_rejections_have_matching_outcomes() {
    for (code, outcome) in [
        ("no_injected_key", "hometown_no_injected_key"),
        ("key_inactive", "hometown_key_inactive"),
    ] {
        let hometown = ScriptedStub::error(401, code);
        let olive = idle();
        let case = run_case(&hometown, &olive, "question", 2);
        case.assert_failed(21, &format!("hometown:{code}"));
        assert_eq!(hometown.count(), 1);
        assert_eq!(case.outcome(), outcome);
    }
}

#[test]
fn tc_ans_05_4xx_without_well_formed_code_uses_status() {
    for status in [400u16, 409, 422] {
        for body in ["", "{}", "not json"] {
            let hometown = ScriptedStub::new(vec![Reply::json(status, body)]);
            let olive = idle();
            let case = run_case(&hometown, &olive, "question", 2);
            case.assert_failed(21, &format!("hometown:{status}"));
            assert_eq!(hometown.count(), 1);
        }
    }
}

#[test]
fn tc_ans_05_hostile_error_bodies_never_reach_output() {
    let long = "a".repeat(65);
    let bodies = [
        serde_json::json!({"error": "bad\ncode-LEAKTEXT"}).to_string(),
        error_body("UPPER-LEAKTEXT"),
        error_body(&long),
        serde_json::json!({"error": 5, "detail": "LEAKTEXT"}).to_string(),
        "<html>LEAKTEXT</html>".to_owned(),
    ];
    for body in bodies {
        let hometown = ScriptedStub::new(vec![Reply::json(422, &body)]);
        let olive = idle();
        let case = run_case(&hometown, &olive, CANARY, 2);
        case.assert_failed(21, "hometown:422");
        assert!(!case.stderr().contains("LEAKTEXT"));
        case.assert_content_free(&["LEAKTEXT", &long]);
        assert_eq!(hometown.count(), 1);
    }
}

fn question_rejected(question: &str) {
    let hometown = ScriptedStub::new(vec![Reply::json(200, ANSWER)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, question, 2);
    case.assert_failed(11, "input:question_length");
    let stats = case.proxy.stats();
    assert_eq!(
        stats.hometown_hits + stats.olive_hits + stats.gateway_attempts,
        0
    );
    assert_eq!(hometown.count(), 0);
    assert!(
        !case.runs.exists() || std::fs::read_dir(&case.runs).unwrap().next().is_none(),
        "run record written"
    );
}

#[test]
fn tc_ans_06_empty_question_rejected() {
    question_rejected("");
}

#[test]
fn tc_ans_06_4001_ascii_rejected() {
    question_rejected(&"a".repeat(4001));
}

#[test]
fn tc_ans_06_4001_scalars_with_non_bmp_rejected() {
    // 4000 two-byte scalars plus one 4-byte scalar: 4001 scalars.
    let question = format!("{}\u{1F600}", "\u{e9}".repeat(4000));
    assert_eq!(question.chars().count(), 4001);
    question_rejected(&question);
}

#[test]
fn tc_ans_06_one_and_4000_scalars_accepted() {
    let max = format!("{}\u{1F600}{}", "\u{e9}".repeat(100), "b".repeat(3899));
    assert_eq!(max.chars().count(), 4000);
    for question in ["x".to_owned(), max] {
        let hometown = ScriptedStub::new(vec![Reply::json(200, ANSWER)]);
        let olive = idle();
        let case = run_case(&hometown, &olive, &question, 2);
        assert_eq!(case.code(), 0, "stderr: {}", case.stderr());
        assert_eq!(hometown.count(), 1);
    }
}

#[test]
fn tc_ans_07_empty_500_retried_once_then_unavailable() {
    let hometown = ScriptedStub::new(vec![Reply::empty(500)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    case.assert_failed(21, "hometown:unavailable");
    assert_eq!(hometown.count(), 2);
    assert_eq!(case.proxy.stats().olive_hits, 0);
    assert_eq!(case.outcome(), "hometown_unavailable");
}

#[test]
fn tc_ans_07_429_rate_limited_not_retried() {
    let hometown = ScriptedStub::error(429, "rate_limited");
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    case.assert_failed(21, "hometown:rate_limited");
    assert_eq!(hometown.count(), 1);
    assert_eq!(case.proxy.stats().hometown_hits, 1);
}

// ---- TC-FAIL ----

#[test]
fn tc_fail_01_500_then_200_succeeds() {
    let hometown = ScriptedStub::new(vec![Reply::empty(500), Reply::json(200, ANSWER)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    assert_eq!(case.code(), 0, "stderr: {}", case.stderr());
    assert_eq!(hometown.count(), 2);
}

#[test]
fn tc_fail_02_500_twice_fails() {
    let hometown = ScriptedStub::new(vec![Reply::empty(500), Reply::empty(500)]);
    let olive = idle();
    let case = run_case(&hometown, &olive, CANARY, 2);
    case.assert_failed(21, "hometown:unavailable");
    assert_eq!(hometown.count(), 2);
    case.assert_content_free(&[]);
}

#[test]
fn tc_fail_03_429_and_409_not_retried() {
    for (status, code) in [(429, "rate_limited"), (409, "run_conflict")] {
        let hometown = ScriptedStub::error(status, code);
        let olive = idle();
        let case = run_case(&hometown, &olive, "question", 2);
        case.assert_failed(21, &format!("hometown:{code}"));
        assert_eq!(hometown.count(), 1, "{code}");
    }
}

#[test]
fn tc_fail_04_timeout_twice_fails_fast() {
    let slow = Reply::json(200, ANSWER).delayed(Duration::from_secs(3));
    let hometown = ScriptedStub::new(vec![slow.clone(), slow]);
    let olive = idle();
    let case = run_case(&hometown, &olive, CANARY, 2);
    case.assert_failed(21, "hometown:unavailable");
    assert_eq!(hometown.count(), 2);
    case.assert_content_free(&[]);
}

/// Real 90 s timeout. TestProxy forwards with reqwest's default 30 s client
/// timeout, so it may answer 502 before Liam's own timeout fires; either way
/// Liam must retry once and exit 21 within 190 s.
#[test]
#[ignore = "long: ~190 s"]
fn tc_fail_04_timeout_100s_real_timeout() {
    let slow = Reply::json(200, ANSWER).delayed(Duration::from_secs(100));
    let hometown = ScriptedStub::new(vec![slow.clone(), slow]);
    let olive = idle();
    let started = std::time::Instant::now();
    let case = run_case(&hometown, &olive, "question", 90);
    assert!(started.elapsed() < Duration::from_secs(190));
    // TestProxy records its own forward timeouts as errors, so assert only Liam's output.
    assert_eq!(case.code(), 21, "stderr: {}", case.stderr());
    assert_eq!(
        case.stderr(),
        "liam: 21 hometown:unavailable
"
    );
    assert_eq!(case.proxy.stats().hometown_hits, 2);
}

#[test]
fn tc_fail_06_olive_500_prints_answer_and_exits_22() {
    let hometown = ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]);
    let olive = ScriptedStub::new(vec![Reply::empty(500)]);
    let case = run_case(&hometown, &olive, CANARY, 2);
    case.assert_failed(22, "olive:unavailable");
    assert_eq!(case.stdout(), "stub answer\n");
    assert_eq!(olive.count(), 1);
    assert_eq!(case.outcome(), "olive_failed");
    case.assert_content_free(&[]);
}

#[test]
fn tc_fail_06_olive_4xx_code_exits_22_without_retry() {
    for (status, code) in [(400, "schema_invalid"), (422, "unknown_action")] {
        let hometown = ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]);
        let olive = ScriptedStub::error(status, code);
        let case = run_case(&hometown, &olive, "question", 2);
        case.assert_failed(22, &format!("olive:{code}"));
        assert_eq!(case.stdout(), "stub answer\n");
        assert_eq!(olive.count(), 1);
    }
}

#[test]
fn tc_fail_06_olive_timeout_exits_22_unavailable() {
    let hometown = ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]);
    let olive = ScriptedStub::new(vec![Reply::json(202, "{}").delayed(Duration::from_secs(3))]);
    let case = run_case(&hometown, &olive, "question", 2);
    case.assert_failed(22, "olive:unavailable");
    assert_eq!(case.stdout(), "stub answer\n");
    assert_eq!(olive.count(), 1);
}

#[test]
fn tc_fail_07_every_failure_is_one_stderr_line() {
    for (status, code) in [(403u16, 21), (401, 21), (500, 21), (429, 21)] {
        let hometown = ScriptedStub::new(vec![Reply::empty(status)]);
        let olive = idle();
        let case = run_case(&hometown, &olive, CANARY, 2);
        assert_eq!(case.code(), code);
        let stderr = case.stderr();
        assert_eq!(stderr.lines().count(), 1, "{stderr}");
        assert!(stderr.starts_with(&format!("liam: {code} ")));
        assert!(stderr.ends_with('\n'));
        case.assert_content_free(&[]);
    }
}

#[test]
fn tc_fail_08_transient_503_codes() {
    let hometown = ScriptedStub::new(vec![
        Reply::json(503, &error_body("trace_not_saved")),
        Reply::json(200, ANSWER),
    ]);
    let olive = idle();
    let case = run_case(&hometown, &olive, "question", 2);
    assert_eq!(case.code(), 0, "stderr: {}", case.stderr());
    assert_eq!(hometown.count(), 2);

    for (status, code) in [(504, "deadline"), (503, "context_unavailable")] {
        let reply = Reply::json(status, &error_body(code));
        let hometown = ScriptedStub::new(vec![reply.clone(), reply]);
        let olive = idle();
        let case = run_case(&hometown, &olive, "question", 2);
        case.assert_failed(21, "hometown:unavailable");
        assert_eq!(hometown.count(), 2, "{code}");
    }
}
