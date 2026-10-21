#![cfg(feature = "stubs")]
#![allow(dead_code)]

use liam::stubs::StubRecordPair;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

pub const AGENT_VERSION: &str = "0.4.1";
// CONF-11 replaces this with the canonical 60a44f00... value.
pub const VOCAB_SHA256: &str = "f3412d6844b50209d0cb2d564af919ee2ac43304b3afb69a919bc65bb6ccb116";

fn slash(value: &Path) -> String {
    value.to_string_lossy().replace('\\', "/")
}

pub fn write_test_config(dir: &Path, hometown_url: &str, olive_url: &str, timeout_seconds: u64) {
    write_test_config_with_runs(dir, hometown_url, olive_url, timeout_seconds, &dir.join("runs"));
}

pub fn write_test_config_with_runs(
    dir: &Path,
    hometown_url: &str,
    olive_url: &str,
    timeout_seconds: u64,
    runs_dir: &Path,
) {
    let root = std::env::current_dir().unwrap();
    let vocab = root.join("contracts/vendored/vocabulary.yaml");
    let catalog = root.join("contracts/catalog-0.4.json");
    let text = format!(
        r#"[agent]
version = "{AGENT_VERSION}"
agent_id = "liam"
[nono]
profile_min_version = 4
record_max_age_seconds = 120
expected_token_arm = ""
[hometown]
base_url = "{hometown_url}"
timeout_seconds = {timeout_seconds}
[olive]
base_url = "{olive_url}"
catalog_path = '{catalog}'
catalog_version = "0.4"
timeout_seconds = {timeout_seconds}
[vocabulary]
path = '{vocab}'
sha256 = "{VOCAB_SHA256}"
[evidence]
runs_dir = '{runs}'
"#,
        catalog = slash(&catalog),
        vocab = slash(&vocab),
        runs = slash(runs_dir),
    );
    fs::write(dir.join("liam.toml"), text).unwrap();
}

/// `exe` is `env!("CARGO_BIN_EXE_liam")` from the calling test crate.
pub fn run_liam(
    exe: &str,
    fixture: &StubRecordPair,
    proxy_url: &str,
    args: &[&str],
    extra_env: &[(&str, &str)],
) -> Output {
    Command::new(exe)
        .current_dir(fixture.directory())
        .args(args)
        .env("LIAM_STUBS", "1")
        .env("LIAM_STUB_PID", "0")
        .env("HTTPS_PROXY", proxy_url)
        .env("HTTP_PROXY", proxy_url)
        .env("NO_PROXY", "")
        .env("NONO_RECEIPT_PATH", fixture.receipt_path())
        .env("NONO_AGENT_RECORD_PATH", fixture.agent_path())
        .envs(extra_env.iter().copied())
        .output()
        .unwrap()
}

pub fn read_run_record(dir: &Path) -> (serde_json::Value, Vec<u8>) {
    let jsons: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(Result::unwrap)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    assert_eq!(jsons.len(), 1, "expected exactly one run record");
    let record = serde_json::from_slice(&fs::read(&jsons[0]).unwrap()).unwrap();
    let log = fs::read(jsons[0].with_extension("jsonl")).unwrap();
    (record, log)
}
