#![cfg(feature = "stubs")]

use liam::stubs::{CaptureProxy, HometownStub, OliveStub, StubRecordPair};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn successful_process_run_makes_one_answer_request_and_writes_content_free_evidence() {
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    let vocab = std::env::current_dir()
        .unwrap()
        .join("contracts/vendored/vocabulary.yaml");
    let catalog = std::env::current_dir()
        .unwrap()
        .join("contracts/catalog-0.4.json");
    let path = |value: &std::path::Path| value.to_string_lossy().replace('\\', "/");
    fs::write(
        fixture.directory().join("liam.toml"),
        format!(
            r#"[agent]
version = "0.4.0"
agent_id = "liam"
[nono]
profile_min_version = 4
record_max_age_seconds = 120
expected_token_arm = ""
[hometown]
base_url = "{}"
timeout_seconds = 2
[olive]
base_url = "http://olive.invalid"
catalog_path = '{}'
catalog_version = "0.4"
timeout_seconds = 2
[vocabulary]
path = '{}'
sha256 = "f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116"
[evidence]
runs_dir = '{}'
"#,
            hometown.base_url(),
            path(&catalog),
            path(&vocab),
            path(&runs)
        ),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .current_dir(fixture.directory())
        .args([
            "--purpose",
            "servicing",
            "--party",
            "CANARY-7f3a",
            "--question",
            "CANARY-7f3a",
        ])
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env("HTTPS_PROXY", proxy.url())
        .env("HTTP_PROXY", proxy.url())
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
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
    let record_path = fs::read_dir(&runs)
        .unwrap()
        .map(Result::unwrap)
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .unwrap()
        .path();
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(&record_path).unwrap()).unwrap();
    let log = fs::read(record_path.with_extension("jsonl")).unwrap();
    assert_eq!(record["log_sha256"], format!("{:x}", Sha256::digest(&log)));
    assert!(!String::from_utf8_lossy(&fs::read(record_path).unwrap()).contains("CANARY-7f3a"));
    assert!(!String::from_utf8_lossy(&log).contains("CANARY-7f3a"));
}

#[test]
fn process_run_submits_at_most_one_catalog_valid_proposal() {
    let hometown = HometownStub::answer_with_proposal();
    let olive = OliveStub::awaiting_approval();
    let proxy = CaptureProxy::forward_two(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    let runs = fixture.directory().join("runs");
    let vocab = std::env::current_dir()
        .unwrap()
        .join("contracts/vendored/vocabulary.yaml");
    let catalog = std::env::current_dir()
        .unwrap()
        .join("contracts/catalog-0.4.json");
    let path = |value: &std::path::Path| value.to_string_lossy().replace('\\', "/");
    fs::write(
        fixture.directory().join("liam.toml"),
        format!(
            r#"[agent]
version = "0.4.0"
agent_id = "liam"
[nono]
profile_min_version = 4
record_max_age_seconds = 120
expected_token_arm = ""
[hometown]
base_url = "{}"
timeout_seconds = 2
[olive]
base_url = "{}"
catalog_path = '{}'
catalog_version = "0.4"
timeout_seconds = 2
[vocabulary]
path = '{}'
sha256 = "f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116"
[evidence]
runs_dir = '{}'
"#,
            hometown.base_url(),
            olive.base_url(),
            path(&catalog),
            path(&vocab),
            path(&runs)
        ),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .current_dir(fixture.directory())
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
        .env("HTTPS_PROXY", proxy.url())
        .env("HTTP_PROXY", proxy.url())
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
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
    let vocab = std::env::current_dir()
        .unwrap()
        .join("contracts/vendored/vocabulary.yaml");
    let catalog = std::env::current_dir()
        .unwrap()
        .join("contracts/catalog-0.4.json");
    let path = |value: &std::path::Path| value.to_string_lossy().replace('\\', "/");
    fs::write(
        fixture.directory().join("liam.toml"),
        format!(
            r#"[agent]
version = "0.4.0"
agent_id = "liam"
[nono]
profile_min_version = 4
record_max_age_seconds = 120
expected_token_arm = ""
[hometown]
base_url = "{}"
timeout_seconds = 2
[olive]
base_url = "http://olive.invalid"
catalog_path = '{}'
catalog_version = "0.4"
timeout_seconds = 2
[vocabulary]
path = '{}'
sha256 = "f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116"
[evidence]
runs_dir = '{}'
"#,
            hometown.base_url(),
            path(&catalog),
            path(&vocab),
            path(&runs)
        ),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .current_dir(fixture.directory())
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
        .env("HTTPS_PROXY", proxy.url())
        .env("HTTP_PROXY", proxy.url())
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
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
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
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
        .env("HTTPS_PROXY", "http://127.0.0.1:50413")
        .env("HTTP_PROXY", "http://127.0.0.1:50413")
        .env("NO_PROXY", "")
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
    let root = std::env::current_dir().unwrap();
    let path = |value: &std::path::Path| value.to_string_lossy().replace('\\', "/");
    fs::write(fixture.directory().join("liam.toml"), format!("[agent]\nversion=\"0.4.0\"\nagent_id=\"liam\"\n[nono]\nprofile_min_version=4\nrecord_max_age_seconds=120\nexpected_token_arm=\"\"\n[hometown]\nbase_url=\"{}\"\ntimeout_seconds=1\n[olive]\nbase_url=\"http://olive.invalid\"\ncatalog_path='{}'\ncatalog_version=\"0.4\"\ntimeout_seconds=1\n[vocabulary]\npath='{}'\nsha256=\"f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116\"\n[evidence]\nruns_dir='{}'\n", hometown.base_url(), path(&root.join("contracts/catalog-0.4.json")), path(&root.join("contracts/vendored/vocabulary.yaml")), path(&fixture.directory().join("runs")))).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .current_dir(fixture.directory())
        .args([
            "--purpose",
            "servicing",
            "--party",
            "opaque",
            "--question",
            "question",
        ])
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env("HTTPS_PROXY", proxy.url())
        .env("HTTP_PROXY", proxy.url())
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
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
    let root = std::env::current_dir().unwrap();
    let path = |value: &std::path::Path| value.to_string_lossy().replace('\\', "/");
    fs::write(fixture.directory().join("liam.toml"), format!("[agent]\nversion=\"0.4.0\"\nagent_id=\"liam\"\n[nono]\nprofile_min_version=4\nrecord_max_age_seconds=120\nexpected_token_arm=\"\"\n[hometown]\nbase_url=\"{}\"\ntimeout_seconds=1\n[olive]\nbase_url=\"http://olive.invalid\"\ncatalog_path='{}'\ncatalog_version=\"0.4\"\ntimeout_seconds=1\n[vocabulary]\npath='{}'\nsha256=\"f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116\"\n[evidence]\nruns_dir='{}'\n", hometown.base_url(), path(&root.join("contracts/catalog-0.4.json")), path(&root.join("contracts/vendored/vocabulary.yaml")), path(&fixture.directory().join("runs")))).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .current_dir(fixture.directory())
        .args([
            "--purpose",
            "servicing",
            "--party",
            "opaque",
            "--question",
            "question",
        ])
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env("HTTPS_PROXY", proxy.url())
        .env("HTTP_PROXY", proxy.url())
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
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
    let root = std::env::current_dir().unwrap();
    let runs_file = fixture.directory().join("runs-file");
    fs::write(&runs_file, "not a directory").unwrap();
    let path = |value: &std::path::Path| value.to_string_lossy().replace('\\', "/");
    fs::write(fixture.directory().join("liam.toml"), format!("[agent]\nversion=\"0.4.0\"\nagent_id=\"liam\"\n[nono]\nprofile_min_version=4\nrecord_max_age_seconds=120\nexpected_token_arm=\"\"\n[hometown]\nbase_url=\"{}\"\ntimeout_seconds=1\n[olive]\nbase_url=\"http://olive.invalid\"\ncatalog_path='{}'\ncatalog_version=\"0.4\"\ntimeout_seconds=1\n[vocabulary]\npath='{}'\nsha256=\"f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116\"\n[evidence]\nruns_dir='{}'\n", hometown.base_url(), path(&root.join("contracts/catalog-0.4.json")), path(&root.join("contracts/vendored/vocabulary.yaml")), path(&runs_file))).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_liam"))
        .current_dir(fixture.directory())
        .args([
            "--purpose",
            "servicing",
            "--party",
            "opaque",
            "--question",
            "question",
        ])
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env("HTTPS_PROXY", proxy.url())
        .env("HTTP_PROXY", proxy.url())
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(30));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "stub answer\n");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim_end(),
        "liam: 30 evidence:write"
    );
    assert_eq!(proxy.captured().request_count, 1);
}
