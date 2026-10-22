#![cfg(feature = "stubs")]

use liam::clients::build_proxy_client;
use liam::clients::hometown::{AnswerRequest, HometownClient, HometownError, HttpHometownClient};
use liam::clients::olive::{HttpOliveClient, OliveClient, OliveError, ProposalRequest};
use liam::stubs::{CaptureProxy, HometownStub, OliveStub};
use serde_json::value::RawValue;

struct RawFields {
    action_type: Box<RawValue>,
    party_ref: Box<RawValue>,
    parameters: Box<RawValue>,
    justification: Box<RawValue>,
}
impl RawFields {
    fn new() -> Self {
        let raw = |s: &str| RawValue::from_string(s.to_string()).unwrap();
        Self {
            action_type: raw(r#""crm.case_note@1""#),
            party_ref: raw(r#""opaque-party""#),
            parameters: raw(r#"{ "note":"x" }"#),
            justification: raw(r#""display only""#),
        }
    }
}
fn rejected(status: u16, reason: &str) -> HometownError {
    HometownError::Rejected {
        status,
        reason: reason.into(),
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires permission to temporarily set HKCU Internet Settings"]
fn explicit_proxy_ignores_the_windows_system_proxy_trap() {
    use std::{
        net::TcpListener,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
        HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_VALUE_TYPE,
    };

    struct Value {
        kind: REG_VALUE_TYPE,
        bytes: Vec<u8>,
    }
    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(Some(0)).collect()
    }
    unsafe fn read_value(key: HKEY, name: &str) -> Option<Value> {
        let name = wide(name);
        let mut kind = 0;
        let mut size = 0;
        if RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null(),
            &mut kind,
            std::ptr::null_mut(),
            &mut size,
        ) != 0
        {
            return None;
        }
        let mut bytes = vec![0; size as usize];
        if RegQueryValueExW(
            key,
            name.as_ptr(),
            std::ptr::null(),
            &mut kind,
            bytes.as_mut_ptr(),
            &mut size,
        ) != 0
        {
            return None;
        }
        bytes.truncate(size as usize);
        Some(Value { kind, bytes })
    }
    unsafe fn restore_value(key: HKEY, name: &str, value: Option<Value>) {
        let name = wide(name);
        if let Some(value) = value {
            let _ = RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                value.kind,
                value.bytes.as_ptr(),
                value.bytes.len() as u32,
            );
        } else {
            let _ = RegDeleteValueW(key, name.as_ptr());
        }
    }
    struct RestoreSettings {
        key: HKEY,
        proxy_enable: Option<Value>,
        proxy_server: Option<Value>,
    }
    impl Drop for RestoreSettings {
        fn drop(&mut self) {
            unsafe {
                restore_value(self.key, "ProxyEnable", self.proxy_enable.take());
                restore_value(self.key, "ProxyServer", self.proxy_server.take());
                let _ = RegCloseKey(self.key);
            }
        }
    }

    let path = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings");
    let mut key = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                0,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                &mut key,
            )
        },
        0,
        "open Internet Settings"
    );
    let restore = RestoreSettings {
        key,
        proxy_enable: unsafe { read_value(key, "ProxyEnable") },
        proxy_server: unsafe { read_value(key, "ProxyServer") },
    };
    let trap = TcpListener::bind("127.0.0.1:0").expect("start trap listener");
    trap.set_nonblocking(true).expect("make trap nonblocking");
    let trap_port = trap.local_addr().expect("trap address").port();
    let trap_seen = Arc::new(AtomicBool::new(false));
    let trap_capture = Arc::clone(&trap_seen);
    let trap_worker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(500);
        while Instant::now() < deadline {
            if trap.accept().is_ok() {
                trap_capture.store(true, Ordering::Relaxed);
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
    unsafe {
        let enable = 1u32.to_le_bytes();
        let name = wide("ProxyEnable");
        assert_eq!(
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                4,
                enable.as_ptr(),
                enable.len() as u32,
            ),
            0
        );
        let server = wide(&format!("http://127.0.0.1:{trap_port}"));
        let name = wide("ProxyServer");
        assert_eq!(
            RegSetValueExW(
                key,
                name.as_ptr(),
                0,
                1,
                server.as_ptr().cast(),
                (server.len() * 2) as u32,
            ),
            0
        );
    }
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let client = HttpHometownClient::new(
        build_proxy_client()
            .expect("build explicit client")
            .client(),
        &proxy.hometown_url(),
        2,
    );
    let answer = client.answer(&AnswerRequest {
        party_ref: "opaque-party",
        question: "question",
        purpose: "servicing",
        run_id: "run",
        nono_session_id: "session",
        step_role: "answer",
    });
    drop(restore);
    trap_worker.join().expect("join trap listener");
    assert!(answer.is_ok());
    assert_eq!(proxy.captured().request_count, 1);
    assert!(!trap_seen.load(Ordering::Relaxed));
}

#[test]
fn hometown_request_has_no_authorization_before_proxy_injection() {
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let transport = build_proxy_client().unwrap();
    let client = HttpHometownClient::new(transport.client(), &proxy.hometown_url(), 2);

    let answer = client
        .answer(&AnswerRequest {
            party_ref: "opaque-party",
            question: "question",
            purpose: "servicing",
            run_id: "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
            nono_session_id: "session-1",
            step_role: "answer",
        })
        .unwrap();

    assert_eq!(answer.trace_id, "trace-stub-1");
    assert!(!proxy.captured().has_authorization);
    assert_eq!(proxy.captured().method, "POST");
    assert!(hometown.captured().body.contains("nono_session_id"));
    assert_eq!(
        hometown.captured().authorization.as_deref(),
        Some("Bearer proxy-sentinel")
    );
    assert_eq!(transport.resolver_calls(), 0);
}

#[test]
fn olive_request_has_no_authorization_before_proxy_injection() {
    let olive = OliveStub::awaiting_approval();
    let proxy = CaptureProxy::forward_once(olive.base_url(), "/v1/proposals");
    let client = HttpOliveClient::new(
        build_proxy_client().unwrap().client(),
        &proxy.olive_url(),
        2,
    );
    let fields = RawFields::new();

    let response = client
        .submit(&ProposalRequest {
            action_type: &fields.action_type,
            party_ref: &fields.party_ref,
            parameters: &fields.parameters,
            justification: &fields.justification,
            run_id: "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
            trace_ref: "trace-1",
        })
        .unwrap();

    assert_eq!(response.status, "awaiting_approval");
    assert!(!proxy.captured().has_authorization);
    let body = olive.captured().body;
    assert!(body.contains("trace_ref"));
    assert!(
        body.contains(r#"{ "note":"x" }"#),
        "raw bytes forwarded verbatim"
    );
    assert_eq!(
        olive.captured().authorization.as_deref(),
        Some("Bearer proxy-sentinel")
    );
}

#[test]
fn hometown_server_error_is_classified_as_retryable_unavailability() {
    let hometown = HometownStub::server_error();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let client = HttpHometownClient::new(
        build_proxy_client().unwrap().client(),
        &proxy.hometown_url(),
        2,
    );
    let result = client.answer(&AnswerRequest {
        party_ref: "opaque-party",
        question: "question",
        purpose: "servicing",
        run_id: "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
        nono_session_id: "session-1",
        step_role: "answer",
    });
    assert_eq!(result.unwrap_err(), HometownError::Unavailable);
}

#[test]
fn hometown_terminal_statuses_have_the_documented_classification() {
    for (status, body, expected) in [
        (401, "{}", rejected(401, "hometown:401")),
        (
            401,
            r#"{"error":"key_inactive"}"#,
            rejected(401, "hometown:key_inactive"),
        ),
        (
            403,
            r#"{"error":"purpose_refused"}"#,
            HometownError::PurposeRefused("hometown:purpose_refused".into()),
        ),
        (
            429,
            r#"{"error":"rate_limited"}"#,
            rejected(429, "hometown:rate_limited"),
        ),
    ] {
        let hometown = HometownStub::status(status, body);
        let proxy = CaptureProxy::forward_once_to(hometown.base_url());
        let client = HttpHometownClient::new(
            build_proxy_client().unwrap().client(),
            &proxy.hometown_url(),
            2,
        );
        let result = client.answer(&AnswerRequest {
            party_ref: "opaque-party",
            question: "question",
            purpose: "servicing",
            run_id: "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
            nono_session_id: "session-1",
            step_role: "answer",
        });
        assert_eq!(result.unwrap_err(), expected);
    }
}

#[test]
fn olive_refusal_and_failures_preserve_status_boundaries() {
    let fields = RawFields::new();
    let refused = OliveStub::status(
        200,
        r#"{"proposal_id":"p-1","status":"refused_policy","reason":"policy"}"#,
    );
    let proxy = CaptureProxy::forward_once(refused.base_url(), "/v1/proposals");
    let client = HttpOliveClient::new(
        build_proxy_client().unwrap().client(),
        &proxy.olive_url(),
        2,
    );
    assert_eq!(
        client
            .submit(&ProposalRequest {
                action_type: &fields.action_type,
                party_ref: &fields.party_ref,
                parameters: &fields.parameters,
                justification: &fields.justification,
                run_id: "run",
                trace_ref: "trace"
            })
            .unwrap()
            .status,
        "refused_policy"
    );
    for status in [400, 500] {
        let olive = OliveStub::status(status, "{}");
        let proxy = CaptureProxy::forward_once(olive.base_url(), "/v1/proposals");
        let client = HttpOliveClient::new(
            build_proxy_client().unwrap().client(),
            &proxy.olive_url(),
            2,
        );
        assert_eq!(
            client
                .submit(&ProposalRequest {
                    action_type: &fields.action_type,
                    party_ref: &fields.party_ref,
                    parameters: &fields.parameters,
                    justification: &fields.justification,
                    run_id: "run",
                    trace_ref: "trace"
                })
                .unwrap_err(),
            if status == 400 {
                OliveError::Rejected {
                    status,
                    reason: "olive:400".into(),
                }
            } else {
                OliveError::Unavailable
            }
        );
    }
}
