#![cfg(feature = "stubs")]

mod common;

use common::{read_run_record, run_liam, write_test_config, write_test_config_with_runs};
use liam::stubs::{CaptureProxy, GatewayTrap, HometownStub, OliveStub, StubRecordPair, TestProxy};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

const EXE: &str = env!("CARGO_BIN_EXE_liam");

#[test]
fn successful_process_run_makes_one_answer_request_and_writes_content_free_evidence() {
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    write_test_config(fixture.directory(), 2);
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &[
            "--purpose",
            "servicing",
            "--party",
            "CANARY-7f3a",
            "--question",
            "CANARY-7f3a",
        ],
        &[],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "stub answer\n");
    assert_eq!(
        hometown.captured().body.matches("nono_session_id").count(),
        1
    );
    let (record, log) = read_run_record(&runs);
    assert_eq!(record["log_sha256"], format!("{:x}", Sha256::digest(&log)));
    assert!(!record.to_string().contains("CANARY-7f3a"));
    assert!(!String::from_utf8_lossy(&log).contains("CANARY-7f3a"));
}

#[test]
fn process_run_submits_at_most_one_catalog_valid_proposal() {
    let hometown = HometownStub::answer_with_proposal();
    let olive = OliveStub::awaiting_approval();
    let proxy = CaptureProxy::forward_two(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(fixture.directory(), 2);
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque-party",
            "--question",
            "question",
        ],
        &[],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(proxy.captured().request_count, 2);
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("Proposal proposal-stub-1: awaiting_approval"));
}

#[test]
fn process_retries_hometown_once_after_a_server_error() {
    let hometown = HometownStub::server_error_then_answer();
    let proxy = CaptureProxy::forward_two_to(hometown.base_url(), "/v1/agent/answer");
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    write_test_config(fixture.directory(), 2);
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque-party",
            "--question",
            "question",
        ],
        &[],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(proxy.captured().request_count, 2);
    assert!(runs.is_dir());
}

#[test]
fn failed_startup_writes_no_evidence() {
    let fixture = StubRecordPair::for_proxy(50413);
    let output = Command::new(EXE)
        .current_dir(std::env::current_dir().unwrap())
        .args([
            "--purpose",
            "servicing",
            "--party",
            "opaque-party",
            "--question",
            "question",
        ])
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env(
            "NONO_SERVICE_HOMETOWN_URL",
            "http://127.0.0.1:50413/svc/hometown",
        )
        .env("NONO_SERVICE_OLIVE_URL", "http://127.0.0.1:50413/svc/olive")
        .env("LOCALAPPDATA", fixture.directory())
        .env(
            "NONO_RECEIPT_PATH",
            fixture.directory().join("missing-receipt.json"),
        )
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(10));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim_end(),
        "liam: 10 receipt:missing"
    );
    assert!(!fixture.directory().join("Fiskroad/liam/runs").exists());
}

#[test]
fn repeated_hometown_server_errors_exit_unavailable_after_two_requests() {
    let hometown = HometownStub::server_error_twice();
    let proxy = CaptureProxy::forward_two_to(hometown.base_url(), "/v1/agent/answer");
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(fixture.directory(), 1);
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque",
            "--question",
            "question",
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(21));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim_end(),
        "liam: 21 hometown:unavailable"
    );
    assert_eq!(proxy.captured().request_count, 2);
}

#[test]
fn repeated_hometown_timeouts_exit_unavailable_after_two_requests_without_olive() {
    let hometown = HometownStub::timeout_twice();
    let proxy = CaptureProxy::forward_two_to(hometown.base_url(), "/v1/agent/answer");
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(fixture.directory(), 1);
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque",
            "--question",
            "question",
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(21));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim_end(),
        "liam: 21 hometown:unavailable"
    );
    assert_eq!(proxy.captured().request_count, 2);
}

#[test]
fn evidence_path_failure_prints_answer_then_exits_with_evidence_write_error() {
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs_file = fixture.directory().join("runs-file");
    fs::write(&runs_file, "not a directory").unwrap();
    write_test_config_with_runs(fixture.directory(), 1, &runs_file);
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque",
            "--question",
            "question",
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(30));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "stub answer\n");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim_end(),
        "liam: 30 evidence:write"
    );
    assert_eq!(proxy.captured().request_count, 1);
}

const ARGS: [&str; 6] = [
    "--purpose",
    "servicing",
    "--party",
    "opaque-party",
    "--question",
    "question",
];

#[test]
fn tc_net_09_service_routes_plain_http_only() {
    let hometown = HometownStub::answer_with_proposal();
    let olive = OliveStub::awaiting_approval();
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(fixture.directory(), 2);
    let output = run_liam(EXE, &fixture, proxy.service_base(), &ARGS, &[]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stats = proxy.stats();
    assert_eq!(stats.paths.len(), 2, "paths: {:?}", stats.paths);
    assert!(stats.paths[0].contains(" /svc/hometown"));
    assert!(stats.paths[1].contains(" /svc/olive"));
    assert!(stats.paths.iter().all(|entry| {
        let (method, path) = entry.split_once(' ').unwrap();
        method != "CONNECT" && (path.starts_with("/svc/hometown") || path.starts_with("/svc/olive"))
    }));
    assert_eq!(stats.connect_attempts, 0);
    assert_eq!(stats.inbound_authorization_count, 0);
    assert_eq!(stats.gateway_attempts, 0);
    assert_eq!(stats.hometown_hits, 1);
    assert_eq!(stats.olive_hits, 1);
}

#[test]
fn proxy_env_vars_are_ignored() {
    const SENTINEL: &str = "SENTINEL-TOKEN";
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let trap = GatewayTrap::new();
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    write_test_config(fixture.directory(), 2);
    let forward = format!("http://nono:{SENTINEL}@{}", trap.addr());
    let output = run_liam(
        EXE,
        &fixture,
        proxy.service_base(),
        &ARGS,
        &[
            ("HTTPS_PROXY", forward.as_str()),
            ("HTTP_PROXY", forward.as_str()),
            ("https_proxy", forward.as_str()),
            ("http_proxy", forward.as_str()),
            ("NO_PROXY", "localhost,127.0.0.1"),
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(trap.hits(), 0);
    assert_eq!(proxy.captured().request_count, 1);
    assert_eq!(proxy.captured().connect_attempts, 0);
    assert!(!String::from_utf8_lossy(&output.stdout).contains(SENTINEL));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(SENTINEL));
    for entry in fs::read_dir(&runs).unwrap() {
        let text = fs::read(entry.unwrap().path()).unwrap();
        assert!(!String::from_utf8_lossy(&text).contains(SENTINEL));
    }
}
