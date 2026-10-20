use liam::{
    clients::{
        build_proxy_client,
        hometown::{HometownError, HttpHometownClient},
        olive::HttpOliveClient,
    },
    config::{expand_local_app_data, load, sha256_hex, validated_proxy},
    evidence::Evidence,
    receipt::{ensure_not_writable, validate_startup, StartupPolicy},
    task::{run, TaskError},
};
use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

fn fail(code: i32, reason: &str) -> ! {
    eprintln!("liam: {code} {reason}");
    std::process::exit(code);
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| fail(10, "agent_record:written_at"))
        .as_secs()
}

#[cfg(feature = "stubs")]
fn launch_process_id() -> u32 {
    if env::var("LIAM_STUBS").as_deref() == Ok("1") {
        if let Ok(value) = env::var("LIAM_STUB_PID") {
            return value.parse().unwrap_or_else(|_| std::process::id());
        }
    }
    std::process::id()
}

#[cfg(not(feature = "stubs"))]
fn launch_process_id() -> u32 {
    std::process::id()
}
fn required_arg(name: &str) -> String {
    let args: Vec<String> = env::args().collect();
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| fail(11, "args:required"))
}
fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| {
        fail(
            10,
            if name == "NONO_RECEIPT_PATH" {
                "receipt:missing"
            } else {
                "agent_record:missing"
            },
        )
    })
}
fn vocabulary_contains_purpose(yaml: &str, purpose: &str) -> bool {
    let mut in_purposes = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed == "purposes:" {
            in_purposes = true;
            continue;
        }
        if in_purposes && !line.starts_with(char::is_whitespace) {
            break;
        }
        if in_purposes && trimmed.strip_prefix("- ") == Some(purpose) {
            return true;
        }
    }
    false
}

fn main() {
    let config_text =
        fs::read_to_string("liam.toml").unwrap_or_else(|_| fail(11, "config:missing"));
    let config = load(&config_text).unwrap_or_else(|reason| fail(11, reason));
    let purpose = required_arg("--purpose");
    let party_ref = required_arg("--party");
    let question = required_arg("--question");
    let vocabulary =
        fs::read(&config.vocabulary.path).unwrap_or_else(|_| fail(11, "vocab:missing"));
    if sha256_hex(&vocabulary) != config.vocabulary.sha256 {
        fail(11, "vocab:sha256");
    }
    if !vocabulary_contains_purpose(
        std::str::from_utf8(&vocabulary).unwrap_or_else(|_| fail(11, "vocab:parse")),
        &purpose,
    ) {
        fail(11, "vocab:purpose");
    }
    let https_proxy = env::var("HTTPS_PROXY").unwrap_or_default();
    let http_proxy = env::var("HTTP_PROXY").unwrap_or_default();
    let proxy = validated_proxy(
        &https_proxy,
        &http_proxy,
        &env::var("NO_PROXY").unwrap_or_default(),
    )
    .unwrap_or_else(|reason| fail(11, reason));
    if !config.nono.record_public_key.is_empty() {
        fail(10, "receipt:signature");
    }
    let receipt_path = required_env("NONO_RECEIPT_PATH");
    let agent_path = required_env("NONO_AGENT_RECORD_PATH");
    ensure_not_writable(std::path::Path::new(&receipt_path), "receipt")
        .unwrap_or_else(|error| fail(10, &error.0));
    ensure_not_writable(std::path::Path::new(&agent_path), "agent_record")
        .unwrap_or_else(|error| fail(10, &error.0));
    let receipt = fs::read_to_string(receipt_path).unwrap_or_else(|_| fail(10, "receipt:missing"));
    let agent = fs::read_to_string(agent_path).unwrap_or_else(|_| fail(10, "agent_record:missing"));
    let stub_arm = if cfg!(feature = "stubs") && env::var("LIAM_STUBS").as_deref() == Ok("1") {
        "WriteRestricted"
    } else {
        &config.nono.expected_token_arm
    };
    let policy = StartupPolicy::for_stub(
        &config.agent.agent_id,
        &purpose,
        stub_arm,
        config.nono.profile_min_version,
        config.nono.record_max_age_seconds,
        config.hometown.timeout_seconds,
        config.olive.timeout_seconds,
    );
    let session = validate_startup(
        &receipt,
        &agent,
        &policy,
        launch_process_id(),
        now(),
        proxy.port,
    )
    .unwrap_or_else(|error| fail(10, &error.0));
    let run_id = Uuid::now_v7().to_string();
    let mut evidence = Evidence::new(
        expand_local_app_data(&config.evidence.runs_dir),
        run_id.clone(),
        now(),
    );
    let transport =
        build_proxy_client(&proxy.url).unwrap_or_else(|_| fail(21, "hometown:unavailable"));
    let hometown = HttpHometownClient::new(
        transport.client(),
        &config.hometown.base_url,
        config.hometown.timeout_seconds,
    );
    let olive = HttpOliveClient::new(
        transport.client(),
        &config.olive.base_url,
        config.olive.timeout_seconds,
    );
    let catalog = fs::read_to_string(&config.olive.catalog_path)
        .unwrap_or_else(|_| fail(22, "olive:catalog"));
    let task = run(
        &hometown,
        &olive,
        &catalog,
        &config.olive.catalog_version,
        &party_ref,
        &question,
        &purpose,
        &run_id,
        &session.session_id,
    )
    .or_else(|error| match error {
        TaskError::Hometown(HometownError::Unavailable) => run(
            &hometown,
            &olive,
            &catalog,
            &config.olive.catalog_version,
            &party_ref,
            &question,
            &purpose,
            &run_id,
            &session.session_id,
        ),
        other => Err(other),
    });
    match task {
        Ok(result) => {
            if !result.context_complete {
                println!("Notice: answer may be incomplete.");
            }
            println!("{}", result.answer);
            if let Some((proposal_id, status)) = &result.proposal {
                println!("Proposal {proposal_id}: {status}");
            }
            if result.suppressed.is_some() {
                evidence.event("proposal.suppressed");
            }
            if evidence
                .finish(
                    &session,
                    &config.agent.agent_id,
                    &config.agent.version,
                    config.nono.profile_min_version,
                    &purpose,
                    &party_ref,
                    "completed",
                    Some((&result.trace_id, result.context_complete)),
                    result
                        .proposal
                        .as_ref()
                        .map(|(id, status)| (id.as_str(), status.as_str())),
                    now(),
                )
                .is_err()
            {
                fail(30, "evidence:write");
            }
        }
        Err(TaskError::Hometown(HometownError::PurposeRefused)) => finish_failure(
            &mut evidence,
            &session,
            &config,
            &purpose,
            &party_ref,
            "purpose_refused",
            20,
            "hometown:purpose_refused",
        ),
        Err(TaskError::Hometown(HometownError::NoInjectedKey)) => finish_failure(
            &mut evidence,
            &session,
            &config,
            &purpose,
            &party_ref,
            "hometown_no_injected_key",
            21,
            "hometown:no_injected_key",
        ),
        Err(TaskError::Hometown(HometownError::KeyInactive)) => finish_failure(
            &mut evidence,
            &session,
            &config,
            &purpose,
            &party_ref,
            "hometown_key_inactive",
            21,
            "hometown:key_inactive",
        ),
        Err(TaskError::Hometown(HometownError::Status(status))) => finish_failure(
            &mut evidence,
            &session,
            &config,
            &purpose,
            &party_ref,
            "hometown_failed",
            21,
            &format!("hometown:{status}"),
        ),
        Err(TaskError::Hometown(_)) => finish_failure(
            &mut evidence,
            &session,
            &config,
            &purpose,
            &party_ref,
            "hometown_unavailable",
            21,
            "hometown:unavailable",
        ),
        Err(TaskError::Olive(_)) => finish_failure(
            &mut evidence,
            &session,
            &config,
            &purpose,
            &party_ref,
            "olive_failed",
            22,
            "olive:unavailable",
        ),
    }
}
#[allow(clippy::too_many_arguments)]
fn finish_failure(
    evidence: &mut Evidence,
    session: &liam::receipt::ValidatedSession,
    config: &liam::config::RuntimeConfig,
    purpose: &str,
    party_ref: &str,
    outcome: &str,
    code: i32,
    reason: &str,
) -> ! {
    if evidence
        .finish(
            session,
            &config.agent.agent_id,
            &config.agent.version,
            config.nono.profile_min_version,
            purpose,
            party_ref,
            outcome,
            None,
            None,
            now(),
        )
        .is_err()
    {
        fail(30, "evidence:write");
    }
    fail(code, reason)
}
