//! Nono record parsing and startup validation.

use crate::expectancy::{accepts, ALL_LAYER_IDS};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum LayerStatus {
    Confirmed,
    EstablishedNotIndependentlyObservable,
    Unconfirmed,
    NotApplicable,
}

#[derive(Debug, Deserialize)]
struct LayerRow {
    id: String,
    status: LayerStatus,
}

#[derive(Debug, Deserialize)]
struct EnforcementReceipt {
    schema_version: u16,
    session_id: String,
    pid: u32,
    entry_path: String,
    token_arm: Option<String>,
    outcome: String,
    layers: Vec<LayerRow>,
}

#[derive(Debug, Deserialize)]
struct AgentSessionRecord {
    schema_version: u16,
    session_id: String,
    agent_id: String,
    purpose: String,
    // Untyped so a non-string key_id reports agent_record:key_id, not parse.
    key_id: serde_json::Value,
    key_expires_at_unix: u64,
    injected_on: Vec<String>,
    direct_gateway: bool,
    profile_version: u32,
    proxy_port: u16,
    written_at_unix: u64,
}

#[derive(Debug)]
pub struct StartupPolicy {
    pub agent_id: String,
    pub purpose: String,
    pub expected_token_arm: String,
    pub profile_min_version: u32,
    pub record_max_age_seconds: u64,
    pub hometown_timeout_seconds: u64,
    pub olive_timeout_seconds: u64,
}

impl StartupPolicy {
    pub fn for_stub(
        agent_id: &str,
        purpose: &str,
        expected_token_arm: &str,
        profile_min_version: u32,
        record_max_age_seconds: u64,
        hometown_timeout_seconds: u64,
        olive_timeout_seconds: u64,
    ) -> Self {
        Self {
            agent_id: agent_id.to_owned(),
            purpose: purpose.to_owned(),
            expected_token_arm: expected_token_arm.to_owned(),
            profile_min_version,
            record_max_age_seconds,
            hometown_timeout_seconds,
            olive_timeout_seconds,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedSession {
    pub session_id: String,
    pub agent_key_id: String,
    pub token_arm: String,
    pub layers_outside_expectancy_scope: Vec<(String, LayerStatus)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StartupError(pub String);

fn receipt_error(field: &str) -> StartupError {
    StartupError(format!("receipt:{field}"))
}

fn agent_error(field: &str) -> StartupError {
    StartupError(format!("agent_record:{field}"))
}

/// The supervisor-owned records must not be writable by Liam. Opening with
/// write access performs an effective-access check without modifying a byte.
pub fn ensure_not_writable(path: &Path, field: &str) -> Result<(), StartupError> {
    match OpenOptions::new().write(true).open(path) {
        Ok(_) => Err(if field == "receipt" {
            receipt_error("writable")
        } else {
            agent_error("writable")
        }),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(_) => Err(if field == "receipt" {
            receipt_error("missing")
        } else {
            agent_error("missing")
        }),
    }
}

pub fn validate_startup(
    receipt_json: &str,
    agent_json: &str,
    policy: &StartupPolicy,
    process_id: u32,
    now_unix: u64,
    proxy_port: u16,
) -> Result<ValidatedSession, StartupError> {
    let receipt: EnforcementReceipt =
        serde_json::from_str(receipt_json).map_err(|_| receipt_error("parse"))?;
    let agent: AgentSessionRecord =
        serde_json::from_str(agent_json).map_err(|_| agent_error("parse"))?;

    if receipt.schema_version != 2 {
        return Err(receipt_error("schema_version"));
    }
    if receipt.session_id.is_empty() {
        return Err(receipt_error("session_id"));
    }
    if receipt.pid != process_id {
        return Err(receipt_error("pid"));
    }
    if receipt.entry_path != "DirectCli" {
        return Err(receipt_error("entry_path"));
    }
    let token_arm = receipt
        .token_arm
        .ok_or_else(|| receipt_error("token_arm"))?;
    if !matches!(
        token_arm.as_str(),
        "Null" | "WriteRestricted" | "LowIlPrimary" | "BrokerLaunch" | "BrokerLaunchNoPty"
    ) || token_arm == "LowIlPrimary"
        || token_arm == "BrokerLaunch"
        || token_arm == "BrokerLaunchNoPty"
        || policy.expected_token_arm.is_empty()
        || token_arm != policy.expected_token_arm
    {
        return Err(receipt_error("token_arm"));
    }
    if receipt.outcome != "Ran" {
        return Err(receipt_error("outcome"));
    }

    let mut seen = HashSet::new();
    if receipt.layers.len() != ALL_LAYER_IDS.len() {
        return Err(receipt_error("layers"));
    }
    let mut outside_scope = Vec::new();
    for row in receipt.layers {
        if !ALL_LAYER_IDS.contains(&row.id.as_str()) || !seen.insert(row.id.clone()) {
            return Err(receipt_error("layers"));
        }
        if !accepts(&row.id, row.status) {
            return Err(receipt_error(&format!("layer:{}", row.id)));
        }
        if matches!(
            row.id.as_str(),
            "RestrictedToken"
                | "MandatoryIntegrityLabel"
                | "DaclSessionSidGrant"
                | "FirewallRulesEgress"
                | "BrokerAuthenticodeTrustGate"
        ) {
            outside_scope.push((row.id, row.status));
        }
    }
    if seen.len() != ALL_LAYER_IDS.len() {
        return Err(receipt_error("layers"));
    }

    if agent.schema_version != 1 {
        return Err(agent_error("schema_version"));
    }
    if agent.session_id != receipt.session_id {
        return Err(agent_error("session_id"));
    }
    if agent.agent_id != policy.agent_id {
        return Err(agent_error("agent_id"));
    }
    if agent.purpose != policy.purpose {
        return Err(agent_error("purpose"));
    }
    let key_id = agent
        .key_id
        .as_str()
        .ok_or_else(|| agent_error("key_id"))?
        .to_owned();
    let key = Uuid::parse_str(&key_id).map_err(|_| agent_error("key_id"))?;
    if key.to_string() != key_id {
        return Err(agent_error("key_id"));
    }
    let minimum_expiry = now_unix
        + 2 * policy
            .hometown_timeout_seconds
            .max(policy.olive_timeout_seconds);
    if agent.key_expires_at_unix < minimum_expiry {
        return Err(agent_error("key_expiry"));
    }
    let injected: HashSet<&str> = agent.injected_on.iter().map(String::as_str).collect();
    if injected.len() != 2 || injected != HashSet::from(["hometown", "olive"]) {
        return Err(agent_error("injected_on"));
    }
    if agent.direct_gateway {
        return Err(agent_error("direct_gateway"));
    }
    if agent.profile_version < policy.profile_min_version {
        return Err(agent_error("profile_version"));
    }
    if agent.proxy_port != proxy_port {
        return Err(agent_error("proxy_port"));
    }
    if agent.written_at_unix > now_unix + 5
        || now_unix.saturating_sub(agent.written_at_unix) > policy.record_max_age_seconds
    {
        return Err(agent_error("written_at"));
    }

    Ok(ValidatedSession {
        session_id: receipt.session_id,
        agent_key_id: key_id,
        token_arm,
        layers_outside_expectancy_scope: outside_scope,
    })
}
