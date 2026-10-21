#![cfg(feature = "stubs")]
//! Key, meta, vocabulary and network TCs (TC-KEY-01..03, TC-META-01/02,
//! TC-LVOC-01/02, TC-NET-05) plus the TC-NET mapping below.
//!
//! TC-NET mapping (no duplicate test added where one exists):
//!   TC-NET-01 -> process_contract.rs successful_process_run_makes_one_answer_request_and_writes_content_free_evidence
//!                (stubs are only reachable through the capturing proxy; request counts are taken at the proxy)
//!   TC-NET-05 -> tc_net_05_proxy_environment_variants below (all seven cases; process_contract.rs
//!                non_loopback_proxy_environment_fails_before_any_request_or_evidence covers one)
//!   TC-NET-06 -> proxy_contract.rs hometown_request_has_no_authorization_before_proxy_injection
//!                (asserts resolver_calls() == 0), plus the DenyAllResolver unit tests in src/clients/mod.rs
//!   TC-NET-07 -> proxy_contract.rs explicit_proxy_ignores_the_windows_system_proxy_trap
//!                (#[ignore]: needs permission to set HKCU Internet Settings; RESULTS row stays partial)
//!
//! TC-KEY-03 does not read the repo-root `.env` file: its contents are
//! deliberately never opened by tooling.
//!
//! Vocabulary pin note: `contracts/vendored/**` carries a `-text` attribute, so
//! checkouts keep the canonical vocabulary bytes (SHA-256 60a44f00...).

mod common;

use common::{read_run_record, run_liam, write_test_config};
use liam::config::sha256_hex;
use liam::stubs::{Reply, ScriptedStub, StubRecordPair, TestProxy};
use serde_json::Value;
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::Output,
};

const EXE: &str = env!("CARGO_BIN_EXE_liam");
const CANONICAL_VOCAB_SHA256: &str =
    "60a44f00db0daafcad3a4e11fe8fcbb2821e1ca7879302bae413b212a0c1e8e7";
const SENTINEL: &str = "proxy-sentinel";

fn repo(path: &str) -> PathBuf {
    std::env::current_dir().unwrap().join(path)
}

fn proposal_answer() -> String {
    serde_json::json!({
        "trace_id": "trace-stub-1",
        "answer": "stub answer",
        "context_complete": true,
        "proposal": {
            "action_type": "crm.case_note@1",
            "party_ref": "opaque-party",
            "parameters": {"note": "stub"},
            "justification": "stub justification"
        }
    })
    .to_string()
}

struct Run {
    output: Output,
    proxy: TestProxy,
    hometown: ScriptedStub,
    olive: ScriptedStub,
    fixture: StubRecordPair,
}

impl Run {
    fn runs(&self) -> PathBuf {
        self.fixture.directory().join("runs")
    }
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }
}

fn run_args(hometown: ScriptedStub, olive: ScriptedStub, args: &[&str]) -> Run {
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(
        fixture.directory(),
        hometown.base_url(),
        olive.base_url(),
        2,
    );
    let output = run_liam(EXE, &fixture, proxy.url(), args, &[]);
    Run {
        output,
        proxy,
        hometown,
        olive,
        fixture,
    }
}

fn run_default(hometown: ScriptedStub, olive: ScriptedStub) -> Run {
    run_args(
        hometown,
        olive,
        &[
            "--purpose",
            "servicing",
            "--party",
            "opaque-party",
            "--question",
            "question",
        ],
    )
}

fn proposal_run() -> Run {
    run_default(
        ScriptedStub::new(vec![Reply::json(200, &proposal_answer())]),
        ScriptedStub::olive_ok(202, "prop-1", "awaiting_approval", None),
    )
}

// ---- TC-KEY ----

#[test]
fn tc_key_01_liam_sends_no_authorization_header() {
    let run = proposal_run();
    assert_eq!(run.output.status.code(), Some(0), "{}", run.stderr());
    let stats = run.proxy.stats();
    assert_eq!(stats.hometown_hits, 1);
    assert_eq!(stats.olive_hits, 1);
    assert_eq!(
        stats.inbound_authorization_count, 0,
        "a request leaving Liam carried Authorization"
    );
    // The stubs see only the credential the proxy injected afterwards.
    for stub in [&run.hometown, &run.olive] {
        let requests = stub.requests();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].authorization_present, "proxy sentinel missing");
    }
}

#[test]
fn tc_key_02_key_rejections_exit_21_after_one_request() {
    for code in ["no_injected_key", "key_inactive"] {
        let run = run_default(
            ScriptedStub::error(401, code),
            ScriptedStub::error(500, "x"),
        );
        assert_eq!(run.output.status.code(), Some(21), "{}", run.stderr());
        assert_eq!(run.stderr(), format!("liam: 21 hometown:{code}\n"));
        assert_eq!(run.hometown.count(), 1, "{code}");
        assert_eq!(run.olive.count(), 0, "{code}");
    }
}

fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

/// Credential-shaped findings in `data`: long hex runs that are not a named
/// SHA-256 pin or a `*sha256` / `party_ref_hash` field value, `Bearer <non-sentinel>`, and
/// `sk-` token prefixes followed by 20+ token characters.
fn key_shaped(data: &[u8], allowed_pins: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if is_hex(data[i]) && (i == 0 || !data[i - 1].is_ascii_alphanumeric()) {
            let start = i;
            while i < data.len() && is_hex(data[i]) {
                i += 1;
            }
            let run = &data[start..i];
            let boundary_ok = i >= data.len() || !data[i].is_ascii_alphanumeric();
            if run.len() >= 32 && boundary_ok {
                let text = String::from_utf8_lossy(run).to_ascii_lowercase();
                let before = &data[start.saturating_sub(40)..start];
                let named_field = String::from_utf8_lossy(before)
                    .to_ascii_lowercase()
                    .contains("sha256")
                    || String::from_utf8_lossy(before).contains("party_ref_hash");
                let pinned = allowed_pins.iter().any(|pin| *pin == text);
                let before_text = String::from_utf8_lossy(before);
                // Named exceptions: rustc commit hash in std source paths and
                // digit-only lookup tables (no hex letters).
                let toolchain_path =
                    before_text.ends_with("rustc/") || before_text.ends_with("rustc\\");
                let digits_only = run.iter().all(u8::is_ascii_digit);
                if toolchain_path || digits_only {
                    continue;
                }
                if !(run.len() == 64 && (named_field || pinned)) {
                    found.push(format!("hex-run:{}", run.len()));
                }
            }
            continue;
        }
        i += 1;
    }
    let text = String::from_utf8_lossy(data);
    let mut rest: &str = &text;
    while let Some(at) = rest.find("Bearer ") {
        let after = &rest[at + 7..];
        if !after.starts_with(SENTINEL) {
            found.push("bearer".to_owned());
        }
        rest = after;
    }
    let mut rest: &str = &text;
    while let Some(at) = rest.find("sk-") {
        let after = &rest[at + 3..];
        let token = after
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
            .count();
        let prefix_ok = at == 0 || !rest.as_bytes()[at - 1].is_ascii_alphanumeric();
        if token >= 20 && prefix_ok {
            found.push("sk-prefix".to_owned());
        }
        rest = after;
    }
    found
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn tc_key_03_scan_finds_no_credential_shaped_values() {
    let run = proposal_run();
    assert_eq!(run.output.status.code(), Some(0), "{}", run.stderr());
    let pins = [common::VOCAB_SHA256, CANONICAL_VOCAB_SHA256];

    let mut files = vec![
        PathBuf::from(EXE),
        repo("liam.toml"),
        run.fixture.directory().join("liam.toml"),
        run.fixture.receipt_path().to_path_buf(),
        run.fixture.agent_path().to_path_buf(),
    ];
    for dir in ["contracts", "profiles", "scripts"] {
        collect_files(&repo(dir), &mut files);
    }
    collect_files(&run.runs(), &mut files);
    let mut scanned = 0;
    for file in &files {
        let Ok(data) = fs::read(file) else { continue };
        scanned += 1;
        let findings = key_shaped(&data, &pins);
        assert!(findings.is_empty(), "{}: {findings:?}", file.display());
    }
    assert!(scanned >= 8, "scan reached too few files: {scanned}");
    // stderr and stdout of the run itself.
    assert!(key_shaped(&run.output.stderr, &pins).is_empty());
    assert!(key_shaped(&run.output.stdout, &pins).is_empty());
}

#[test]
fn tc_key_03_negative_control_scanner_detects_key_shapes() {
    // Synthetic shapes built at runtime; none is a real credential.
    let hex = "ab".repeat(20);
    let sk = format!("sk-{}", "Ab1".repeat(10));
    let bearer = format!("Bearer {}", "zz");
    assert!(!key_shaped(hex.as_bytes(), &[]).is_empty());
    assert!(!key_shaped(sk.as_bytes(), &[]).is_empty());
    assert!(!key_shaped(bearer.as_bytes(), &[]).is_empty());
    assert!(key_shaped(format!("Bearer {SENTINEL}").as_bytes(), &[]).is_empty());
    let pin = "f".repeat(64);
    assert!(key_shaped(pin.as_bytes(), &[&pin]).is_empty());
    assert!(key_shaped(format!("\"party_ref_sha256\":\"{pin}\"").as_bytes(), &[]).is_empty());
    assert!(!key_shaped(pin.as_bytes(), &[]).is_empty());
}

// ---- TC-META ----

#[test]
fn tc_meta_01_run_and_session_ids_join_across_records() {
    let run = proposal_run();
    assert_eq!(run.output.status.code(), Some(0), "{}", run.stderr());
    let hometown: Value = serde_json::from_str(&run.hometown.requests()[0].body).unwrap();
    let olive: Value = serde_json::from_str(&run.olive.requests()[0].body).unwrap();
    let run_id = hometown["run_id"].as_str().unwrap();
    let parsed = uuid::Uuid::parse_str(run_id).unwrap();
    assert_eq!(parsed.get_version_num(), 7);
    assert_eq!(hometown["nono_session_id"], "stub-session-1");
    assert_eq!(hometown["step_role"], "answer");
    assert_eq!(olive["run_id"], run_id);
    assert_eq!(olive["trace_ref"], "trace-stub-1");
    let (record, _) = read_run_record(&run.runs());
    assert_eq!(record["run_id"], run_id);
    assert_eq!(record["answer"]["trace_id"], "trace-stub-1");
    assert_eq!(
        record["nono"]["agent_key_id"],
        "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b"
    );
}

#[test]
fn tc_meta_02_exactly_one_hometown_request() {
    let run = proposal_run();
    assert_eq!(run.output.status.code(), Some(0), "{}", run.stderr());
    assert_eq!(run.hometown.count(), 1);
    assert_eq!(run.proxy.stats().hometown_hits, 1);
}

// ---- TC-LVOC ----

fn pinned_sha256() -> String {
    let text = fs::read_to_string(repo("liam.toml")).unwrap();
    let value: toml::Value = toml::from_str(&text).unwrap();
    value["vocabulary"]["sha256"].as_str().unwrap().to_owned()
}

#[test]
fn tc_lvoc_01_unknown_purpose_exits_11_before_any_request() {
    // The working-tree file must match the pin liam.toml carries.
    let bytes = fs::read(repo("contracts/vendored/vocabulary.yaml")).unwrap();
    assert_eq!(sha256_hex(&bytes), pinned_sha256());

    let run = run_args(
        ScriptedStub::new(vec![Reply::empty(500)]),
        ScriptedStub::new(vec![Reply::empty(500)]),
        &["--purpose", "invented", "--party", "p", "--question", "q"],
    );
    assert_eq!(run.output.status.code(), Some(11));
    assert_eq!(run.stderr(), "liam: 11 vocab:purpose\n");
    let stats = run.proxy.stats();
    assert_eq!(stats.hometown_hits + stats.olive_hits, 0);
    assert!(!run.runs().exists());
}

#[test]
fn tc_lvoc_02_modified_vocabulary_exits_11_before_any_request() {
    let hometown = ScriptedStub::new(vec![Reply::empty(500)]);
    let olive = ScriptedStub::new(vec![Reply::empty(500)]);
    let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
    let fixture = StubRecordPair::for_proxy(proxy.port());
    write_test_config(
        fixture.directory(),
        hometown.base_url(),
        olive.base_url(),
        2,
    );
    let mut bytes = fs::read(repo("contracts/vendored/vocabulary.yaml")).unwrap();
    bytes.push(b'#');
    let modified = fixture.directory().join("vocabulary.yaml");
    fs::write(&modified, bytes).unwrap();
    let config_path = fixture.directory().join("liam.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let original = repo("contracts/vendored/vocabulary.yaml")
        .to_string_lossy()
        .replace('\\', "/");
    assert!(config.contains(&original));
    let replacement = modified.to_string_lossy().replace('\\', "/");
    fs::write(&config_path, config.replace(&original, &replacement)).unwrap();
    let output = run_liam(
        EXE,
        &fixture,
        proxy.url(),
        &["--purpose", "servicing", "--party", "p", "--question", "q"],
        &[],
    );
    assert_eq!(output.status.code(), Some(11));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "liam: 11 vocab:sha256\n"
    );
    let stats = proxy.stats();
    assert_eq!(stats.hometown_hits + stats.olive_hits, 0);
    assert_eq!(hometown.count(), 0);
    assert!(!fixture.directory().join("runs").exists());
}

#[test]
fn tc_lvoc_canonical_hash_pinned() {
    assert_eq!(pinned_sha256(), CANONICAL_VOCAB_SHA256);
    let bytes = fs::read(repo("contracts/vendored/vocabulary.yaml")).unwrap();
    assert_eq!(sha256_hex(&bytes), CANONICAL_VOCAB_SHA256);
}

// ---- TC-NET-05 ----

#[test]
fn tc_net_05_proxy_environment_variants() {
    // A listener on every alternative address; none may see a connection.
    let v4 = TcpListener::bind("127.0.0.1:0").unwrap();
    v4.set_nonblocking(true).unwrap();
    let v6 = TcpListener::bind("[::1]:0").ok();
    if let Some(v6) = &v6 {
        v6.set_nonblocking(true).unwrap();
    }
    let p = v4.local_addr().unwrap().port();
    let p6 = v6.as_ref().map_or(p, |l| l.local_addr().unwrap().port());

    let loopback = format!("http://127.0.0.1:{p}");
    #[allow(clippy::type_complexity)]
    let cases: Vec<(String, Vec<(&str, String)>, &str)> = vec![
        (
            format!("http://localhost:{p}"),
            vec![],
            "env:proxy_not_loopback",
        ),
        (
            format!("http://[::1]:{p6}"),
            vec![],
            "env:proxy_not_loopback",
        ),
        (
            format!("http://proxy.lab:{p}"),
            vec![],
            "env:proxy_not_loopback",
        ),
        (
            format!("https://127.0.0.1:{p}"),
            vec![],
            "env:proxy_not_loopback",
        ),
        (
            format!("http://u:pw@127.0.0.1:{p}"),
            vec![],
            "env:proxy_not_loopback",
        ),
        (
            loopback.clone(),
            vec![(
                "HTTP_PROXY",
                format!("http://127.0.0.1:{}", p.wrapping_add(1)),
            )],
            "env:proxy_not_loopback",
        ),
        (
            loopback,
            vec![("NO_PROXY", "hometown.lab".to_owned())],
            "env:no_proxy_nonempty",
        ),
    ];
    for (https_proxy, extra, reason) in cases {
        let hometown = ScriptedStub::new(vec![Reply::empty(500)]);
        let olive = ScriptedStub::new(vec![Reply::empty(500)]);
        let fixture = StubRecordPair::for_proxy(p);
        write_test_config(
            fixture.directory(),
            hometown.base_url(),
            olive.base_url(),
            2,
        );
        let extra_refs: Vec<(&str, &str)> = extra.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let output = run_liam(
            EXE,
            &fixture,
            &https_proxy,
            &["--purpose", "servicing", "--party", "p", "--question", "q"],
            &extra_refs,
        );
        assert_eq!(output.status.code(), Some(11), "{https_proxy}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!("liam: 11 {reason}\n"),
            "{https_proxy} {extra:?}"
        );
        assert_eq!(hometown.count() + olive.count(), 0);
        assert!(!fixture.directory().join("runs").exists());
    }
    assert!(v4.accept().is_err(), "listener saw a connection");
    if let Some(v6) = &v6 {
        assert!(v6.accept().is_err(), "ipv6 listener saw a connection");
    }
}
