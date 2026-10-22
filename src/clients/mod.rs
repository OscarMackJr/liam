pub mod hometown;
pub mod olive;

use reqwest::{
    blocking::Client,
    dns::{Name, Resolve, Resolving},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Default)]
pub struct DenyAllResolver {
    calls: AtomicUsize,
}

impl DenyAllResolver {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl Resolve for DenyAllResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Box::pin(async { Err("liam DNS resolution is forbidden".into()) })
    }
}

pub struct ServiceClient {
    client: Client,
    resolver: Arc<DenyAllResolver>,
}

impl ServiceClient {
    pub fn client(&self) -> Client {
        self.client.clone()
    }

    pub fn resolver_calls(&self) -> usize {
        self.resolver.calls()
    }
}

pub fn build_service_client() -> Result<ServiceClient, reqwest::Error> {
    let resolver = Arc::new(DenyAllResolver::default());
    let client = Client::builder()
        .no_proxy()
        // One call is one request (design §3.3, §7.4, D12): never follow a redirect.
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(Arc::clone(&resolver))
        .build()?;
    Ok(ServiceClient { client, resolver })
}

#[derive(serde::Deserialize)]
struct ErrorBody {
    error: String,
}

/// Returns the `error` code from a `{"error": "<code>"}` body only if it
/// matches ^[a-z][a-z0-9_]{0,63}$; anything else yields None (D-06).
pub fn error_code(body: &[u8]) -> Option<String> {
    let parsed: ErrorBody = serde_json::from_slice(body).ok()?;
    guard_code(&parsed.error).then_some(parsed.error)
}

pub(crate) fn guard_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
}

/// Liam-side guard for upstream identifiers that reach the run record, the log
/// and stdout (hometown `trace_id`, olive `proposal_id`):
/// `^[A-Za-z0-9._:-]{1,128}$`. Pending confirmation in S-HT-1 / S-OL-1.
pub(crate) fn id_ok(id: &str) -> bool {
    (1..=128).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
}

/// `service:code` when the body carries a guarded code, else `service:status`.
pub fn reason(service: &str, status: u16, body: &[u8]) -> String {
    match error_code(body) {
        Some(code) => format!("{service}:{code}"),
        None => format!("{service}:{status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{build_service_client, error_code, id_ok, reason, DenyAllResolver};

    #[test]
    fn id_guard_accepts_only_the_identifier_pattern() {
        for ok in [
            "t",
            "trace-stub-1",
            "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
            "A.b_c:D-9",
        ] {
            assert!(id_ok(ok), "{ok}");
        }
        assert!(id_ok(&"a".repeat(128)));
        for bad in [
            String::new(),
            "a".repeat(129),
            "has space".into(),
            "slash/x".into(),
            "esc\u{1b}[2J".into(),
            "cr\r".into(),
            "caf\u{e9}".into(),
            "{\"x\":1}".into(),
        ] {
            assert!(!id_ok(&bad), "{bad:?}");
        }
    }

    #[test]
    fn error_code_accepts_only_guarded_codes() {
        assert_eq!(
            error_code(br#"{"error":"rate_limited"}"#).as_deref(),
            Some("rate_limited")
        );
        let long = format!(r#"{{"error":"{}"}}"#, "a".repeat(65));
        let max = format!(r#"{{"error":"{}"}}"#, "a".repeat(64));
        assert!(error_code(long.as_bytes()).is_none());
        assert!(error_code(max.as_bytes()).is_some());
        for bad in [
            &br#"{"error":5}"#[..],
            br#"{"other":"x"}"#,
            b"",
            br#"{"error":"Bad-Code"}"#,
            br#"{"error":"1abc"}"#,
            br#"{"error":"a b"}"#,
            br#"{"error":"a\nb"}"#,
            br#"{"error":""}"#,
        ] {
            assert!(error_code(bad).is_none());
        }
        assert_eq!(reason("hometown", 403, b""), "hometown:403");
        assert_eq!(
            reason("olive", 400, br#"{"error":"bad_thing"}"#),
            "olive:bad_thing"
        );
    }

    use reqwest::dns::Resolve;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };

    #[test]
    fn deny_all_resolver_starts_with_no_calls() {
        assert_eq!(DenyAllResolver::default().calls(), 0);
    }

    #[test]
    fn loopback_proxy_client_is_built_with_a_resolver_that_has_not_run() {
        let client = build_service_client().expect("build client");
        assert_eq!(client.resolver_calls(), 0);
    }

    #[test]
    fn deny_all_resolver_fails_closed_on_every_lookup() {
        let resolver = DenyAllResolver::default();
        let mut resolution = resolver.resolve("hometown.test".parse().expect("valid name"));
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Pin::new(&mut resolution).poll(&mut context),
            Poll::Ready(Err(_))
        ));
        assert_eq!(resolver.calls(), 1);
    }
}
