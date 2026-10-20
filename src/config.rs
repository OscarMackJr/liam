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

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatedProxy {
    pub url: String,
    pub port: u16,
}

pub fn load(input: &str) -> Result<RuntimeConfig, &'static str> {
    toml::from_str(input).map_err(|_| "config:parse")
}
pub fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}
pub fn validated_proxy(
    https_proxy: &str,
    http_proxy: &str,
    no_proxy: &str,
) -> Result<ValidatedProxy, &'static str> {
    if https_proxy.is_empty() || http_proxy.is_empty() {
        return Err("env:no_proxy");
    }
    if !no_proxy.is_empty() {
        return Err("env:no_proxy_nonempty");
    }
    if https_proxy != http_proxy {
        return Err("env:proxy_not_loopback");
    }
    let port = https_proxy
        .strip_prefix("http://127.0.0.1:")
        .ok_or("env:proxy_not_loopback")?
        .parse()
        .map_err(|_| "env:proxy_not_loopback")?;
    if port == 0 || https_proxy != format!("http://127.0.0.1:{port}") {
        return Err("env:proxy_not_loopback");
    }
    Ok(ValidatedProxy {
        url: https_proxy.to_owned(),
        port,
    })
}
pub fn expand_local_app_data(path: &str) -> String {
    std::env::var("LOCALAPPDATA")
        .map(|value| path.replace("%LOCALAPPDATA%", &value))
        .unwrap_or_else(|_| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{sha256_hex, validated_proxy};
    #[test]
    fn sha256_matches_the_standard_empty_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn accepts_only_matching_ipv4_loopback_proxy_urls_and_empty_no_proxy() {
        let proxy = validated_proxy("http://127.0.0.1:50413", "http://127.0.0.1:50413", "")
            .expect("valid loopback proxy");
        assert_eq!(proxy.port, 50413);
        assert_eq!(proxy.url, "http://127.0.0.1:50413");
    }

    #[test]
    fn rejects_non_loopback_or_mismatched_proxy_environment() {
        for value in [
            "http://localhost:50413",
            "http://[::1]:50413",
            "http://proxy.lab:50413",
            "https://127.0.0.1:50413",
            "http://user:password@127.0.0.1:50413",
            "http://127.0.0.1:50413/path",
        ] {
            assert_eq!(
                validated_proxy(value, value, ""),
                Err("env:proxy_not_loopback")
            );
        }
        assert_eq!(
            validated_proxy("http://127.0.0.1:50413", "http://127.0.0.1:50414", ""),
            Err("env:proxy_not_loopback")
        );
        assert_eq!(
            validated_proxy(
                "http://127.0.0.1:50413",
                "http://127.0.0.1:50413",
                "hometown.lab"
            ),
            Err("env:no_proxy_nonempty")
        );
    }
}
