//! CI-only HTTP fixtures. They are feature-gated and absent from default
//! release builds. The proxy simulates nono-proxy injection at its outbound
//! boundary, never in Liam's client.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tiny_http::{Response, Server};

#[derive(Clone, Default)]
pub struct ProxyCapture {
    pub has_authorization: bool,
    pub method: String,
    pub request_count: u32,
}
#[derive(Clone, Default)]
pub struct HometownCapture {
    pub authorization: Option<String>,
    pub body: String,
}

pub struct CaptureProxy {
    url: String,
    capture: Arc<Mutex<ProxyCapture>>,
    _thread: thread::JoinHandle<()>,
}
impl CaptureProxy {
    pub fn forward_once_to(target_base: &str) -> Self {
        Self::forward_once(target_base, "/v1/agent/answer")
    }
    pub fn forward_once(target_base: &str, path: &str) -> Self {
        let server = Server::http("127.0.0.1:0").expect("start capture proxy");
        let url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let target = format!("{}{}", target_base.trim_end_matches('/'), path);
        let capture = Arc::new(Mutex::new(ProxyCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive proxied request");
            let has_authorization = request
                .headers()
                .iter()
                .any(|header| header.field.equiv("Authorization"));
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.has_authorization = has_authorization;
            captured.method = request.method().as_str().to_owned();
            captured.request_count += 1;
            drop(captured);
            let mut body = Vec::new();
            request
                .as_reader()
                .read_to_end(&mut body)
                .expect("read request body");
            let response = reqwest::blocking::Client::new()
                .post(target)
                .header("Authorization", "Bearer proxy-sentinel")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .expect("forward request");
            let status = response.status().as_u16();
            let body = response.bytes().expect("read forwarded response");
            request
                .respond(Response::from_data(body).with_status_code(status))
                .expect("respond to Liam");
        });
        Self {
            url,
            capture,
            _thread: worker,
        }
    }
    pub fn forward_two(hometown_base: &str, olive_base: &str) -> Self {
        let server = Server::http("127.0.0.1:0").expect("start capture proxy");
        let url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let hometown_target = format!("{}/v1/agent/answer", hometown_base.trim_end_matches('/'));
        let olive_target = format!("{}/v1/proposals", olive_base.trim_end_matches('/'));
        let capture = Arc::new(Mutex::new(ProxyCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            for target in [hometown_target, olive_target] {
                let mut request = server.recv().expect("receive proxied request");
                let has_authorization = request
                    .headers()
                    .iter()
                    .any(|header| header.field.equiv("Authorization"));
                let mut captured = thread_capture.lock().expect("capture lock");
                captured.has_authorization |= has_authorization;
                captured.method = request.method().as_str().to_owned();
                captured.request_count += 1;
                drop(captured);
                let mut body = Vec::new();
                request
                    .as_reader()
                    .read_to_end(&mut body)
                    .expect("read request body");
                let response = reqwest::blocking::Client::new()
                    .post(target)
                    .header("Authorization", "Bearer proxy-sentinel")
                    .header("Content-Type", "application/json")
                    .body(body)
                    .send()
                    .expect("forward request");
                let status = response.status().as_u16();
                let body = response.bytes().expect("read forwarded response");
                let _ = request.respond(Response::from_data(body).with_status_code(status));
            }
        });
        Self {
            url,
            capture,
            _thread: worker,
        }
    }
    pub fn forward_two_to(target_base: &str, path: &str) -> Self {
        let server = Server::http("127.0.0.1:0").expect("start capture proxy");
        let url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let target = format!("{}{}", target_base.trim_end_matches('/'), path);
        let capture = Arc::new(Mutex::new(ProxyCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            for _ in 0..2 {
                let mut request = server.recv().expect("receive proxied request");
                let has_authorization = request
                    .headers()
                    .iter()
                    .any(|header| header.field.equiv("Authorization"));
                let mut captured = thread_capture.lock().expect("capture lock");
                captured.has_authorization |= has_authorization;
                captured.method = request.method().as_str().to_owned();
                captured.request_count += 1;
                drop(captured);
                let mut body = Vec::new();
                request
                    .as_reader()
                    .read_to_end(&mut body)
                    .expect("read request body");
                let target = target.clone();
                thread::spawn(move || {
                    let response = reqwest::blocking::Client::new()
                        .post(target)
                        .header("Authorization", "Bearer proxy-sentinel")
                        .header("Content-Type", "application/json")
                        .body(body)
                        .send()
                        .expect("forward request");
                    let status = response.status().as_u16();
                    let body = response.bytes().expect("read forwarded response");
                    let _ = request.respond(Response::from_data(body).with_status_code(status));
                });
            }
        });
        Self {
            url,
            capture,
            _thread: worker,
        }
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn port(&self) -> u16 {
        self.url
            .rsplit_once(':')
            .expect("proxy port")
            .1
            .parse()
            .expect("numeric proxy port")
    }
    pub fn captured(&self) -> ProxyCapture {
        self.capture.lock().expect("capture lock").clone()
    }
}

/// Supervisor-shaped schema-2/schema-1 fixture pair for process tests. It
/// holds no credential: `key_id` is an opaque UUID join identifier only.
pub struct StubRecordPair {
    dir: PathBuf,
    receipt: PathBuf,
    agent: PathBuf,
}
impl StubRecordPair {
    pub fn for_proxy(proxy_port: u16) -> Self {
        let dir = std::env::temp_dir().join(format!("liam-records-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("create fixture directory");
        let receipt = dir.join("receipt.json");
        let agent = dir.join("agent.json");
        let layers = serde_json::json!([
            {"id":"RestrictedToken","status":"Unconfirmed"},{"id":"MandatoryIntegrityLabel","status":"NotApplicable"},{"id":"AppContainerProfile","status":"Confirmed"},{"id":"DaclSessionSidGrant","status":"NotApplicable"},{"id":"DaclPackageSidGrant","status":"Confirmed"},{"id":"DaclAncestorTraverse","status":"Confirmed"},{"id":"DaclAncestorReadAttrs","status":"Confirmed"},{"id":"WfpEgressFilters","status":"Confirmed"},{"id":"FirewallRulesEgress","status":"NotApplicable"},{"id":"MinifilterAbsence","status":"NotApplicable"},{"id":"JobObjectContainment","status":"Confirmed"},{"id":"BrokerAuthenticodeTrustGate","status":"NotApplicable"},{"id":"InterpreterCoverageGate","status":"Confirmed"},{"id":"AgentKeyInjection","status":"Confirmed"}
        ]);
        fs::write(&receipt, serde_json::to_vec(&serde_json::json!({"schema_version":2,"session_id":"stub-session-1","pid":0,"entry_path":"DirectCli","token_arm":"WriteRestricted","outcome":"Ran","layers":layers})).expect("serialize receipt")).expect("write receipt");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_secs();
        fs::write(&agent, serde_json::to_vec(&serde_json::json!({"schema_version":1,"session_id":"stub-session-1","agent_id":"liam","purpose":"servicing","key_id":"0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b","key_expires_at_unix":now + 3600,"injected_on":["hometown","olive"],"direct_gateway":false,"profile_version":4,"proxy_port":proxy_port,"written_at_unix":now})).expect("serialize agent record")).expect("write agent record");
        for path in [&receipt, &agent] {
            let mut permissions = fs::metadata(path).expect("fixture metadata").permissions();
            permissions.set_readonly(true);
            fs::set_permissions(path, permissions).expect("protect fixture");
        }
        Self {
            dir,
            receipt,
            agent,
        }
    }
    pub fn receipt_path(&self) -> &Path {
        &self.receipt
    }
    pub fn agent_path(&self) -> &Path {
        &self.agent
    }
    pub fn directory(&self) -> &Path {
        &self.dir
    }
}

pub struct HometownStub {
    base_url: String,
    capture: Arc<Mutex<HometownCapture>>,
    _thread: thread::JoinHandle<()>,
}
impl HometownStub {
    pub fn status(status: u16, response_body: &str) -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let response_body = response_body.to_owned();
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive hometown request");
            let authorization = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            request
                .as_reader()
                .read_to_string(&mut body)
                .expect("read hometown body");
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            drop(captured);
            request
                .respond(Response::from_string(response_body).with_status_code(status))
                .expect("respond from hometown");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn answer_without_proposal() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive hometown request");
            let authorization = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            request
                .as_reader()
                .read_to_string(&mut body)
                .expect("read hometown body");
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            request.respond(Response::from_string(r#"{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":null}"#).with_status_code(200)).expect("respond from hometown");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn answer_with_proposal() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive hometown request");
            let authorization = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            request
                .as_reader()
                .read_to_string(&mut body)
                .expect("read hometown body");
            let party_ref = serde_json::from_str::<serde_json::Value>(&body)
                .expect("parse answer request")["party_ref"]
                .as_str()
                .expect("party ref")
                .to_owned();
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            drop(captured);
            let response = serde_json::json!({"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":{"action_type":"crm.case_note@1","party_ref":party_ref,"parameters":{"note":"stub"},"justification":"stub justification"}});
            request
                .respond(Response::from_string(response.to_string()).with_status_code(200))
                .expect("respond from hometown");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn server_error() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive hometown request");
            let authorization = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            request
                .as_reader()
                .read_to_string(&mut body)
                .expect("read hometown body");
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            drop(captured);
            request
                .respond(Response::from_string("temporary failure").with_status_code(500))
                .expect("respond from hometown");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn server_error_then_answer() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            let first = server.recv().expect("receive hometown retry request");
            first
                .respond(Response::from_string("temporary failure").with_status_code(500))
                .expect("respond server error");
            let mut second = server.recv().expect("receive retried hometown request");
            let authorization = second
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            second
                .as_reader()
                .read_to_string(&mut body)
                .expect("read hometown body");
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            drop(captured);
            second.respond(Response::from_string(r#"{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":null}"#).with_status_code(200)).expect("respond from hometown");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn server_error_twice() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let worker = thread::spawn(move || {
            for _ in 0..2 {
                let request = server.recv().expect("receive hometown request");
                request
                    .respond(Response::from_string("temporary failure").with_status_code(500))
                    .expect("respond server error");
            }
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn timeout_twice() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start hometown stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let worker = thread::spawn(move || {
            for _ in 0..2 {
                let request = server.recv().expect("receive hometown request");
                thread::sleep(Duration::from_secs(2));
                let _ = request.respond(
                    Response::from_string(
                        r#"{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":null}"#,
                    )
                    .with_status_code(200),
                );
            }
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn captured(&self) -> HometownCapture {
        self.capture.lock().expect("capture lock").clone()
    }
}

pub struct OliveStub {
    base_url: String,
    capture: Arc<Mutex<HometownCapture>>,
    _thread: thread::JoinHandle<()>,
}
impl OliveStub {
    pub fn status(status: u16, response_body: &str) -> Self {
        let server = Server::http("127.0.0.1:0").expect("start olive stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let response_body = response_body.to_owned();
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive olive request");
            let authorization = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            request
                .as_reader()
                .read_to_string(&mut body)
                .expect("read olive body");
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            drop(captured);
            request
                .respond(Response::from_string(response_body).with_status_code(status))
                .expect("respond from olive");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn awaiting_approval() -> Self {
        let server = Server::http("127.0.0.1:0").expect("start olive stub");
        let base_url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(HometownCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let worker = thread::spawn(move || {
            let mut request = server.recv().expect("receive olive request");
            let authorization = request
                .headers()
                .iter()
                .find(|header| header.field.equiv("Authorization"))
                .map(|header| header.value.as_str().to_owned());
            let mut body = String::new();
            request
                .as_reader()
                .read_to_string(&mut body)
                .expect("read olive body");
            let mut captured = thread_capture.lock().expect("capture lock");
            captured.authorization = authorization;
            captured.body = body;
            request
                .respond(
                    Response::from_string(
                        r#"{"proposal_id":"proposal-stub-1","status":"awaiting_approval"}"#,
                    )
                    .with_status_code(202),
                )
                .expect("respond from olive");
        });
        Self {
            base_url,
            capture,
            _thread: worker,
        }
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn captured(&self) -> HometownCapture {
        self.capture.lock().expect("capture lock").clone()
    }
}
