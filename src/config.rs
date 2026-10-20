//! Configuration and integrity helpers. Configuration contains policy and locations, never credentials.

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct RuntimeConfig {
    pub agent: AgentConfig,
    pub nono: NonoConfig,
    pub hometown: ServiceConfig,
    pub olive: OliveConfig,
    pub vocabulary: VocabularyConfig,
    pub evidence: EvidenceConfig,
}
#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    pub version: String,
    pub agent_id: String,
}
#[derive(Debug, Deserialize)]
pub struct NonoConfig {
    pub profile_min_version: u32,
    pub record_max_age_seconds: u64,
    #[serde(default)]
    pub record_public_key: String,
    #[serde(default)]
    pub expected_token_arm: String,
}
#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
}
#[derive(Debug, Deserialize)]
pub struct OliveConfig {
    pub base_url: String,
    pub catalog_path: String,
    pub catalog_version: String,
    pub timeout_seconds: u64,
}
#[derive(Debug, Deserialize)]
pub struct VocabularyConfig {
    pub path: String,
    pub sha256: String,
}
#[derive(Debug, Deserialize)]
pub struct EvidenceConfig {
    pub runs_dir: String,
}

pub fn load(input: &str) -> Result<RuntimeConfig, &'static str> {
    toml::from_str(input).map_err(|_| "config:parse")
}
pub fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}
pub fn proxy_port(https_proxy: &str, http_proxy: &str) -> Result<u16, &'static str> {
    if https_proxy.is_empty() || http_proxy.is_empty() {
        return Err("env:no_proxy");
    }
    https_proxy
        .rsplit_once(':')
        .ok_or("env:proxy_port")?
        .1
        .parse()
        .map_err(|_| "env:proxy_port")
}
pub fn expand_local_app_data(path: &str) -> String {
    std::env::var("LOCALAPPDATA")
        .map(|value| path.replace("%LOCALAPPDATA%", &value))
        .unwrap_or_else(|_| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;
    #[test]
    fn sha256_matches_the_standard_empty_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
