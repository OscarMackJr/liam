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
/// Resolves the evidence `runs_dir` before any network request. The only
/// placeholder is `%LOCALAPPDATA%`; it must expand from a non-empty
/// `local_app_data`, no `%` may remain, and the result must be absolute, so
/// evidence never lands in a folder relative to the working directory.
pub fn resolve_runs_dir(path: &str, local_app_data: Option<&str>) -> Result<String, &'static str> {
    let expanded = if path.contains("%LOCALAPPDATA%") {
        match local_app_data {
            Some(value) if !value.is_empty() => path.replace("%LOCALAPPDATA%", value),
            _ => return Err("config:runs_dir"),
        }
    } else {
        path.to_owned()
    };
    if expanded.contains('%') || !std::path::Path::new(&expanded).is_absolute() {
        return Err("config:runs_dir");
    }
    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::{resolve_runs_dir, sha256_hex, validated_proxy};

    #[test]
    fn runs_dir_must_expand_to_an_absolute_path() {
        let path = r"%LOCALAPPDATA%\Fiskroad\liam\runs";
        assert_eq!(
            resolve_runs_dir(path, Some(r"C:\Users\u\AppData\Local")).unwrap(),
            r"C:\Users\u\AppData\Local\Fiskroad\liam\runs"
        );
        for (path, local) in [
            (r"%LOCALAPPDATA%\runs", None),
            (r"%LOCALAPPDATA%\runs", Some("")),
            (r"%LOCALAPPDATA%\runs", Some("relative")),
            ("runs", Some(r"C:\x")),
            (r"%TEMP%\runs", Some(r"C:\x")),
            (r"C:\%OTHER%\runs", Some(r"C:\x")),
        ] {
            assert_eq!(
                resolve_runs_dir(path, local),
                Err("config:runs_dir"),
                "{path} {local:?}"
            );
        }
        assert!(resolve_runs_dir("C:/abs/runs", None).is_ok());
    }
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
