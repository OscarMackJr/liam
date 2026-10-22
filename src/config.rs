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
    pub timeout_seconds: u64,
}
#[derive(Debug, Deserialize)]
pub struct OliveConfig {
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
pub struct ValidatedServiceUrl {
    pub url: String,
    pub port: u16,
}

pub fn load(input: &str) -> Result<RuntimeConfig, &'static str> {
    toml::from_str(input).map_err(|_| "config:parse")
}
pub fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}
/// Strict manual parse of a `NONO_SERVICE_*_URL` value: exactly
/// `http://127.0.0.1:<port><route>`. Reasons are fixed strings and never
/// carry the value (content-free).
pub fn validated_service_url(
    value: &str,
    route: &str,
) -> Result<ValidatedServiceUrl, &'static str> {
    if value.is_empty() {
        return Err("env:service_url_missing");
    }
    if value.contains('?') || value.contains('#') {
        return Err("env:service_url_not_loopback");
    }
    let rest = value
        .strip_prefix("http://127.0.0.1:")
        .ok_or("env:service_url_not_loopback")?;
    let (segment, remainder) = match rest.find('/') {
        Some(index) => rest.split_at(index),
        None => (rest, ""),
    };
    let port: u16 = segment
        .parse()
        .map_err(|_| "env:service_url_not_loopback")?;
    if port == 0 || format!("{port}") != segment {
        return Err("env:service_url_not_loopback");
    }
    if remainder != route {
        return Err("env:service_url_route");
    }
    Ok(ValidatedServiceUrl {
        url: value.to_owned(),
        port,
    })
}
pub fn check_service_port(url_port: u16, record_port: u16) -> Result<(), &'static str> {
    if url_port == record_port {
        Ok(())
    } else {
        Err("env:service_url_port")
    }
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
    use super::{check_service_port, resolve_runs_dir, sha256_hex, validated_service_url};

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
    fn accepts_exact_service_urls() {
        let url = validated_service_url("http://127.0.0.1:50413/svc/hometown", "/svc/hometown")
            .expect("valid");
        assert_eq!(url.port, 50413);
        assert_eq!(url.url, "http://127.0.0.1:50413/svc/hometown");
        assert!(validated_service_url("http://127.0.0.1:1/svc/olive", "/svc/olive").is_ok());
    }

    #[test]
    fn missing_service_url_is_reported() {
        assert_eq!(
            validated_service_url("", "/svc/hometown"),
            Err("env:service_url_missing")
        );
    }

    #[test]
    fn rejects_non_loopback_service_urls() {
        for value in [
            "http://localhost:50413/svc/hometown",
            "http://[::1]:50413/svc/hometown",
            "http://svc.lab:50413/svc/hometown",
            "https://127.0.0.1:50413/svc/hometown",
            "http://u:pw@127.0.0.1:50413/svc/hometown",
            "http://127.0.0.1:50413/svc/hometown?x=1",
            "http://127.0.0.1:50413/svc/hometown#f",
            "http://127.0.0.1:50413/svc/hometown?",
            "http://127.0.0.1:50413/svc/hometown#",
            "http://127.1:50413/svc/hometown",
            "http://2130706433:50413/svc/hometown",
            "http://0x7f.0.0.1:50413/svc/hometown",
            "http://127.0.0.1/svc/hometown",
            "http://127.0.0.1:abc/svc/hometown",
            "http://127.0.0.1:0/svc/hometown",
            "http://127.0.0.1:05/svc/hometown",
            "http://127.0.0.1:70000/svc/hometown",
            "HTTP://127.0.0.1:50413/svc/hometown",
            "http://127.0.0.1:+50413/svc/hometown",
            "http://127.0.0.1: 50413/svc/hometown",
            "http://127.0.0.1:50413\n/svc/hometown",
            "http://127.0.0.1:５０４１３/svc/hometown",
            "http://127.0.0.1:50413@evil/svc/hometown",
        ] {
            assert_eq!(
                validated_service_url(value, "/svc/hometown"),
                Err("env:service_url_not_loopback"),
                "{value}"
            );
        }
    }

    #[test]
    fn rejects_wrong_routes() {
        for value in [
            "http://127.0.0.1:50413/svc/olive",
            "http://127.0.0.1:50413/svc/hometown/",
            "http://127.0.0.1:50413/svc/hometown/x",
            "http://127.0.0.1:50413//svc/hometown",
            "http://127.0.0.1:50413/other",
            "http://127.0.0.1:50413",
            "http://127.0.0.1:50413/svc/hometownX",
            "http://127.0.0.1:50413/svc/%2e/hometown",
            "http://127.0.0.1:50413/svc/hometow%6e",
            "http://127.0.0.1:50413/svc/hometown\n",
            "http://127.0.0.1:50413/SVC/hometown",
        ] {
            assert_eq!(
                validated_service_url(value, "/svc/hometown"),
                Err("env:service_url_route"),
                "{value}"
            );
        }
    }

    #[test]
    fn service_port_must_match_the_record() {
        assert_eq!(check_service_port(50413, 50413), Ok(()));
        assert_eq!(
            check_service_port(50413, 50414),
            Err("env:service_url_port")
        );
    }

    #[test]
    fn liam_toml_has_no_service_base_url() {
        let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/liam.toml"))
            .expect("liam.toml");
        assert!(!text.contains("base_url"));
    }
}
