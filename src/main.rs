use liam::{
    clients::{
        build_proxy_client,
        hometown::{
            AnswerRequest, AnswerResponse, HometownClient, HometownError, HttpHometownClient,
        },
        olive::{HttpOliveClient, OliveError},
    },
    config::{expand_local_app_data, load, sha256_hex, validated_proxy},
    evidence::{Evidence, ProposalOutcome},
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
fn all_args() -> Vec<String> {
    env::args_os()
        .map(|arg| {
            arg.into_string()
                .unwrap_or_else(|_| fail(11, "args:required"))
        })
        .collect()
}
fn required_arg(args: &[String], name: &str) -> String {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
        .unwrap_or_else(|| fail(11, "args:required"))
}

/// Retries only `hometown.answer`, once, on 5xx/timeout with the same request.
struct RetryOnce<'a>(&'a dyn HometownClient);
impl HometownClient for RetryOnce<'_> {
    fn answer(&self, request: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError> {
        match self.0.answer(request) {
            Err(HometownError::Unavailable) => self.0.answer(request),
            other => other,
        }
    }
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
    let args = all_args();
    let purpose = required_arg(&args, "--purpose");
    let party_ref = required_arg(&args, "--party");
    let question = required_arg(&args, "--question");
    if !(1..=4000).contains(&question.chars().count()) {
        fail(11, "input:question_length");
    }
    let config_text =
        fs::read_to_string("liam.toml").unwrap_or_else(|_| fail(11, "config:missing"));
    let config = load(&config_text).unwrap_or_else(|reason| fail(11, reason));
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
    #[cfg(feature = "stubs")]
    if liam::stubs::gateway_attempt_requested() {
        let outcome = liam::stubs::gateway_attempt(&transport.client());
        evidence.event_with(
            liam::stubs::GATEWAY_EVENT,
            serde_json::json!({ "outcome": outcome }),
        );
    }
    let retrying = RetryOnce(&hometown);
    let task = run(
        &retrying,
        &olive,
        &catalog,
        &config.olive.catalog_version,
        &party_ref,
        &question,
        &purpose,
        &run_id,
        &session.session_id,
    );
    match task {
        Ok(result) => {
            if !result.context_complete {
                println!("Notice: answer may be incomplete.");
            }
            println!("{}", result.answer);
            if let Some((proposal_id, status)) = &result.proposal {
                println!("Proposal {proposal_id}: {status}");
            }
            answer_received(
                &mut evidence,
                &result.trace_id,
                result.context_complete,
                result.has_proposal,
                result.latency_ms,
            );
            if result.validated {
                evidence.event("proposal.validated");
            }
            if let Some(reason) = result.suppressed {
                evidence.event_with(
                    "proposal.suppressed",
                    serde_json::json!({ "reason": reason }),
                );
            }
            for _ in 0..result.limit_suppressed {
                evidence.event_with(
                    "proposal.suppressed",
                    serde_json::json!({ "reason": "limit" }),
                );
            }
            let mut refused_reason: Option<String> = None;
            if let Some(submitted) = &result.submitted {
                evidence.event_with(
                    "proposal.submitted",
                    serde_json::json!({
                        "proposal_id": submitted.proposal_id,
                        "status": submitted.status,
                        "reason": submitted.reason,
                        "http_status": submitted.http_status,
                    }),
                );
                if submitted.status == "refused_policy" {
                    let reason = submitted.reason.clone().unwrap_or_default();
                    evidence.event_with(
                        "proposal.refused",
                        serde_json::json!({
                            "reason": reason,
                            "investigate": reason == "trace_proposal_mismatch",
                        }),
                    );
                    refused_reason = Some(reason);
                }
            }
            let outcome = result
                .proposal
                .as_ref()
                .map(|(id, status)| ProposalOutcome {
                    proposal_id: id.as_str(),
                    status: status.as_str(),
                    reason: refused_reason.as_deref(),
                });
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
                    outcome,
                    now(),
                )
                .is_err()
            {
                fail(30, "evidence:write");
            }
        }
        Err(error) => {
            let (outcome, code, reason) = map_error(&error);
            let mut answer_ids = None;
            if let TaskError::Olive {
                answer,
                context_complete,
                trace_id,
                latency_ms,
                ..
            } = &error
            {
                if !context_complete {
                    println!("Notice: answer may be incomplete.");
                }
                println!("{answer}");
                answer_received(
                    &mut evidence,
                    trace_id,
                    *context_complete,
                    true,
                    *latency_ms,
                );
                evidence.event("proposal.validated");
                answer_ids = Some((trace_id.as_str(), *context_complete));
            }
            if evidence
                .finish(
                    &session,
                    &config.agent.agent_id,
                    &config.agent.version,
                    config.nono.profile_min_version,
                    &purpose,
                    &party_ref,
                    outcome,
                    answer_ids,
                    None,
                    now(),
                )
                .is_err()
            {
                fail(30, "evidence:write");
            }
            fail(code, &reason)
        }
    }
}

/// `answer.received` (design §3.3 step 2): identifiers, flags and a duration only.
fn answer_received(
    evidence: &mut Evidence,
    trace_id: &str,
    context_complete: bool,
    has_proposal: bool,
    latency_ms: u64,
) {
    evidence.event_with(
        "answer.received",
        serde_json::json!({
            "trace_id": trace_id,
            "context_complete": context_complete,
            "has_proposal": has_proposal,
            "latency_ms": latency_ms,
        }),
    );
}

/// One mapping from task failures to (run outcome, exit code, reason line).
fn map_error(error: &TaskError) -> (&'static str, i32, String) {
    match error {
        TaskError::Hometown(error) => {
            let (code, reason) = match error {
                HometownError::PurposeRefused(reason) => (20, reason.clone()),
                HometownError::Rejected { reason, .. } => (21, reason.clone()),
                HometownError::Unavailable | HometownError::InvalidResponse => {
                    (21, "hometown:unavailable".to_string())
                }
            };
            (error.outcome(), code, reason)
        }
        TaskError::Catalog => ("olive_failed", 22, "olive:catalog".to_string()),
        TaskError::Olive { error, .. } => {
            let reason = match error {
                OliveError::Rejected { reason, .. } => reason.clone(),
                _ => "olive:unavailable".to_string(),
            };
            ("olive_failed", 22, reason)
        }
    }
}
