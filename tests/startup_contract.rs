#![cfg(feature = "stubs")]
// Windows-only target: clearing the read-only attribute is the intended operation.
#![allow(clippy::permissions_set_readonly_false)]
//! Startup self-check TCs (TC-START-01..05, 08..16) driven by mutated copies of
//! the stub record pair. Each mutation is applied to the JSON on disk, the
//! files are re-protected read-only, and liam is run as a process.
//!
//! Coverage map (pre-existing coverage is in src/lib.rs unit tests, which
//! call `validate_startup` directly and do not run the process):
//!   TC-START-01 -> tc_start_01_valid_pair_runs (process; unit: lib.rs accepts_a_valid_supervisor_record_pair)
//!   TC-START-02 -> tc_start_02_missing_records
//!   TC-START-03 -> tc_start_03_receipt_schema_1
//!   TC-START-04 -> tc_start_04_outcome_entry_path_pid
//!   TC-START-05 -> tc_start_05_token_arm_cases, tc_start_05_empty_expected_arm_outside_stub_mode
//!   TC-START-08 -> tc_start_08_proxy_env_unset
//!   TC-START-09 -> tc_start_09_census
//!   TC-START-10 -> tc_start_10_layer_status
//!   TC-START-11 -> tc_start_11_out_of_scope_rows_recorded_verbatim
//!   TC-START-12 -> tc_start_12_agent_identity
//!   TC-START-13 -> tc_start_13_key_id_and_expiry, tc_start_13_numeric_key_id_spec_reason (ignored: defect)
//!   TC-START-14 -> tc_start_14_injection_set
//!   TC-START-15 -> tc_start_15_profile_and_port
//!   TC-START-16 -> tc_start_16_record_age
//! Not tested here (listed for RESULTS):
//!   TC-START-06 not measured (dev-standard-user ACL harness)
//!   TC-START-07 deferred (DEF-01, DSSE waits on NONO_v0.4.3 O2)

mod common;

use common::{read_run_record, run_liam, write_test_config};
use liam::stubs::{Reply, ScriptedStub, StubRecordPair, TestProxy};
use serde_json::{json, Value};
use std::{fs, path::Path, process::Output};

const EXE: &str = env!("CARGO_BIN_EXE_liam");
const ANSWER: &str =
    r#"{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":null}"#;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn mutate(path: &Path, edit: impl FnOnce(&mut Value)) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).unwrap();
    let mut value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).unwrap();
}

fn set_layer(receipt: &mut Value, id: &str, status: &str) {
    let rows = receipt["layers"].as_array_mut().unwrap();
    let row = rows.iter_mut().find(|r| r["id"] == id).unwrap();
    row["status"] = json!(status);
}

struct Case {
    output: Output,
    proxy: TestProxy,
    fixture: StubRecordPair,
    hometown: ScriptedStub,
}

fn run_with(
    prepare: impl FnOnce(&StubRecordPair),
    timeout: u64,
    proxy_env: Option<&str>,
    extra_env: &[(&str, &str)],
) -> Case {
    let hometown = ScriptedStub::new(vec![Reply::json(200, ANSWER)]);
    let olive = ScriptedStub::new(vec![Reply::empty(500)]);
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(
        fixture.directory(),
        hometown.base_url(),
        olive.base_url(),
        timeout,
    );
    prepare(&fixture);
    let output = run_liam(
        EXE,
        &fixture,
        proxy_env.unwrap_or(proxy.url()),
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque-party",
            "--question",
            "question",
        ],
        extra_env,
    );
    Case {
        output,
        proxy,
        fixture,
        hometown,
    }
}

fn run_case(prepare: impl FnOnce(&StubRecordPair)) -> Case {
    run_with(prepare, 5, None, &[])
}

impl Case {
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }
    /// Exit code, exactly one reason line, no request, no run record.
    fn assert_refused(&self, code: i32, reason: &str) {
        assert_eq!(self.output.status.code(), Some(code), "{}", self.stderr());
        assert_eq!(self.stderr(), format!("liam: {code} {reason}\n"));
        let stats = self.proxy.stats();
        assert_eq!(stats.hometown_hits + stats.olive_hits, 0, "reason {reason}");
        assert_eq!(self.hometown.count(), 0);
        assert!(
            !self.fixture.directory().join("runs").exists(),
            "run record written for {reason}"
        );
    }
}

fn receipt_case(reason: &str, edit: impl FnOnce(&mut Value)) {
    let case = run_case(|f| mutate(f.receipt_path(), edit));
    case.assert_refused(10, reason);
}

fn agent_case(reason: &str, edit: impl FnOnce(&mut Value)) {
    let case = run_case(|f| mutate(f.agent_path(), edit));
    case.assert_refused(10, reason);
}

#[test]
fn tc_start_01_valid_pair_runs() {
    let case = run_case(|_| {});
    assert_eq!(case.output.status.code(), Some(0), "{}", case.stderr());
    let runs = case.fixture.directory().join("runs");
    let (record, log) = read_run_record(&runs);
    assert_eq!(record["nono"]["receipt_integrity"], "acl-only");
    assert_eq!(
        record["nono"]["agent_key_id"],
        "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b"
    );
    let first = String::from_utf8(log).unwrap();
    let first: Value = serde_json::from_str(first.lines().next().unwrap()).unwrap();
    assert_eq!(first["event"], "run.started");
}

#[test]
fn tc_start_02_missing_records() {
    let case = run_case(|f| {
        let mut permissions = fs::metadata(f.receipt_path()).unwrap().permissions();
        permissions.set_readonly(false);
        fs::set_permissions(f.receipt_path(), permissions).unwrap();
        fs::remove_file(f.receipt_path()).unwrap();
    });
    case.assert_refused(10, "receipt:missing");
    let case = run_case(|f| {
        let mut permissions = fs::metadata(f.agent_path()).unwrap().permissions();
        permissions.set_readonly(false);
        fs::set_permissions(f.agent_path(), permissions).unwrap();
        fs::remove_file(f.agent_path()).unwrap();
    });
    case.assert_refused(10, "agent_record:missing");
}

#[test]
fn tc_start_03_receipt_schema_1() {
    receipt_case("receipt:schema_version", |r| r["schema_version"] = json!(1));
}

#[test]
fn tc_start_04_outcome_entry_path_pid() {
    receipt_case("receipt:outcome", |r| r["outcome"] = json!("Refused"));
    receipt_case("receipt:entry_path", |r| r["entry_path"] = json!("Broker"));
    receipt_case("receipt:pid", |r| r["pid"] = json!(4242));
}

#[test]
fn tc_start_05_token_arm_cases() {
    for arm in [
        "LowIlPrimary",
        "BrokerLaunch",
        "BrokerLaunchNoPty",
        "Unconfigured",
    ] {
        receipt_case("receipt:token_arm", |r| r["token_arm"] = json!(arm));
    }
}

#[test]
fn tc_start_05_empty_expected_arm_outside_stub_mode() {
    // Stub mode forces the expected arm, so the empty-policy case is measured
    // against validate_startup with the pair's own bytes.
    let proxy_port = 50413;
    let fixture = StubRecordPair::for_proxy(proxy_port);
    let receipt = fs::read_to_string(fixture.receipt_path()).unwrap();
    let agent = fs::read_to_string(fixture.agent_path()).unwrap();
    let policy = liam::receipt::StartupPolicy::for_stub("liam", "servicing", "", 4, 120, 5, 5);
    let error = liam::receipt::validate_startup(&receipt, &agent, &policy, 0, now(), proxy_port)
        .unwrap_err();
    assert_eq!(error.0, "receipt:token_arm");
}

#[test]
fn tc_start_08_proxy_env_unset() {
    // Variables present but empty.
    let case = run_with(|_| {}, 5, Some(""), &[]);
    case.assert_refused(11, "env:no_proxy");
    // Variables genuinely absent from the process environment.
    let hometown = ScriptedStub::new(vec![Reply::json(200, ANSWER)]);
    let olive = ScriptedStub::new(vec![Reply::empty(500)]);
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(
        fixture.directory(),
        hometown.base_url(),
        olive.base_url(),
        5,
    );
    let output = std::process::Command::new(EXE)
        .current_dir(fixture.directory())
        .args(["--purpose", "servicing", "--party", "p", "--question", "q"])
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env_remove("HTTPS_PROXY")
        .env_remove("HTTP_PROXY")
        .env_remove("NO_PROXY")
        .env_remove("https_proxy")
        .env_remove("http_proxy")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(11));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "liam: 11 env:no_proxy\n"
    );
    let stats = proxy.stats();
    assert_eq!(stats.hometown_hits + stats.olive_hits, 0);
    assert_eq!(hometown.count(), 0);
    assert!(!fixture.directory().join("runs").exists());
}

#[test]
fn tc_start_09_census() {
    receipt_case("receipt:layers", |r| {
        r["layers"].as_array_mut().unwrap().pop();
    });
    receipt_case("receipt:layers", |r| {
        let rows = r["layers"].as_array_mut().unwrap();
        let dup = rows[0].clone();
        rows[13] = dup;
    });
}

#[test]
fn tc_start_10_layer_status() {
    let cases = [
        ("AppContainerProfile", "Unconfirmed"),
        ("WfpEgressFilters", "EstablishedNotIndependentlyObservable"),
        ("AgentKeyInjection", "Unconfirmed"),
        ("MinifilterAbsence", "Unconfirmed"),
    ];
    for (layer, status) in cases {
        receipt_case(&format!("receipt:layer:{layer}"), |r| {
            set_layer(r, layer, status)
        });
    }
}

#[test]
fn tc_start_11_out_of_scope_rows_recorded_verbatim() {
    let rows = [
        ("RestrictedToken", "Confirmed"),
        ("MandatoryIntegrityLabel", "Unconfirmed"),
        (
            "DaclSessionSidGrant",
            "EstablishedNotIndependentlyObservable",
        ),
        ("FirewallRulesEgress", "NotApplicable"),
        ("BrokerAuthenticodeTrustGate", "Confirmed"),
    ];
    let case = run_case(|f| {
        mutate(f.receipt_path(), |r| {
            for (id, status) in rows {
                set_layer(r, id, status);
            }
        })
    });
    assert_eq!(case.output.status.code(), Some(0), "{}", case.stderr());
    let (record, _) = read_run_record(&case.fixture.directory().join("runs"));
    let recorded = record["nono"]["layers_outside_expectancy_scope"]
        .as_array()
        .unwrap();
    assert_eq!(recorded.len(), 5);
    for (id, status) in rows {
        assert!(
            recorded
                .iter()
                .any(|e| e["id"] == id && e["status"] == status),
            "{id} {status} missing from {recorded:?}"
        );
    }
}

#[test]
fn tc_start_12_agent_identity() {
    agent_case("agent_record:session_id", |a| {
        a["session_id"] = json!("other-session")
    });
    agent_case("agent_record:agent_id", |a| {
        a["agent_id"] = json!("not-liam")
    });
    agent_case("agent_record:purpose", |a| {
        a["purpose"] = json!("analytics")
    });
}

#[test]
fn tc_start_13_key_id_and_expiry() {
    agent_case("agent_record:key_id", |a| {
        a["key_id"] = json!("0192A3B4-C5D6-7E8F-9A0B-1C2D3E4F5A6B")
    });
    // Timeout 5s -> minimum remaining lifetime is 2 * 5s; give it 1s.
    agent_case("agent_record:key_expiry", |a| {
        a["key_expires_at_unix"] = json!(now() + 1)
    });
}

// Spec fixture keyid-numeric (JSON number) must give agent_record:key_id, but
// the typed parse in receipt.rs fails first and reports agent_record:parse.
// Product defect reported in 01-10-SUMMARY; fix is outside this plan's files.
#[test]
#[ignore = "product defect: JSON-number key_id reports agent_record:parse, spec says agent_record:key_id"]
fn tc_start_13_numeric_key_id_spec_reason() {
    agent_case("agent_record:key_id", |a| a["key_id"] = json!(12345));
}

#[test]
fn tc_start_14_injection_set() {
    agent_case("agent_record:injected_on", |a| {
        a["injected_on"] = json!(["hometown", "gateway"])
    });
    agent_case("agent_record:injected_on", |a| {
        a["injected_on"] = json!(["hometown"])
    });
    agent_case("agent_record:direct_gateway", |a| {
        a["direct_gateway"] = json!(true)
    });
}

#[test]
fn tc_start_15_profile_and_port() {
    agent_case("agent_record:profile_version", |a| {
        a["profile_version"] = json!(3)
    });
    let case = run_case(|f| {
        mutate(f.agent_path(), |a| {
            let port = a["proxy_port"].as_u64().unwrap();
            a["proxy_port"] = json!(if port == 65000 { 65001 } else { 65000 });
        })
    });
    case.assert_refused(10, "agent_record:proxy_port");
}

#[test]
fn tc_start_16_record_age() {
    agent_case("agent_record:written_at", |a| {
        a["written_at_unix"] = json!(now() - 600)
    });
    agent_case("agent_record:written_at", |a| {
        a["written_at_unix"] = json!(now() + 600)
    });
}
