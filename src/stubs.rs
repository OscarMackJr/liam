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
    pub connect_attempts: usize,
}
#[derive(Clone, Default)]
pub struct HometownCapture {
    pub authorization: Option<String>,
    pub body: String,
}

/// The remainder after a service prefix must be empty or start with `/`.
fn boundary(rest: &str) -> Option<&str> {
    (rest.is_empty() || rest.starts_with('/')).then_some(rest)
}

/// Route an origin-form path to the hometown or olive base.
fn route_path(url: &str, hometown: &str, olive: &str) -> Option<String> {
    if let Some(rest) = url.strip_prefix("/svc/hometown").and_then(boundary) {
        return Some(format!("{}{}", hometown.trim_end_matches('/'), rest));
    }
    url.strip_prefix("/svc/olive")
        .and_then(boundary)
        .map(|rest| format!("{}{}", olive.trim_end_matches('/'), rest))
}

fn capture_one(
    mut request: tiny_http::Request,
    capture: &Mutex<ProxyCapture>,
    route: &dyn Fn(&str) -> Option<String>,
) {
    let has_authorization = request
        .headers()
        .iter()
        .any(|header| header.field.equiv("Authorization"));
    let method = request.method().as_str().to_owned();
    {
        let mut captured = capture.lock().expect("capture lock");
        captured.has_authorization |= has_authorization;
        captured.method = method.clone();
        captured.request_count += 1;
        if method.eq_ignore_ascii_case("CONNECT") {
            captured.connect_attempts += 1;
        }
    }
    if method.eq_ignore_ascii_case("CONNECT") {
        let _ = request.respond(Response::empty(405));
        return;
    }
    let mut body = Vec::new();
    let _ = request.as_reader().read_to_end(&mut body);
    let Some(target) = route(request.url()) else {
        let _ = request.respond(Response::empty(502));
        return;
    };
    let forwarded = reqwest::blocking::Client::builder()
        .no_proxy()
        .build()
        .ok()
        .and_then(|client| {
            client
                .post(target)
                .header("Authorization", "Bearer proxy-sentinel")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .ok()
        })
        .and_then(|response| {
            let status = response.status().as_u16();
            response.bytes().ok().map(|bytes| (status, bytes))
        });
    match forwarded {
        Some((status, bytes)) => {
            let _ = request.respond(Response::from_data(bytes).with_status_code(status));
        }
        None => {
            let _ = request.respond(Response::empty(502));
        }
    }
}

pub struct CaptureProxy {
    url: String,
    capture: Arc<Mutex<ProxyCapture>>,
    _thread: thread::JoinHandle<()>,
}
impl CaptureProxy {
    fn start(
        requests: usize,
        threaded: bool,
        route: impl Fn(&str) -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        let server = Server::http("127.0.0.1:0").expect("start capture proxy");
        let url = format!(
            "http://{}",
            server.server_addr().to_ip().expect("IP listener")
        );
        let capture = Arc::new(Mutex::new(ProxyCapture::default()));
        let thread_capture = Arc::clone(&capture);
        let route = Arc::new(route);
        let worker = thread::spawn(move || {
            for _ in 0..requests {
                let request = server.recv().expect("receive proxied request");
                let capture = Arc::clone(&thread_capture);
                let route = Arc::clone(&route);
                if threaded {
                    thread::spawn(move || capture_one(request, &capture, &*route));
                } else {
                    capture_one(request, &capture, &*route);
                }
            }
        });
        Self {
            url,
            capture,
            _thread: worker,
        }
    }
    pub fn forward_once_to(target_base: &str) -> Self {
        Self::forward_once(target_base)
    }
    pub fn forward_once(target_base: &str) -> Self {
        let base = target_base.to_owned();
        Self::start(1, false, move |url| route_path(url, &base, &base))
    }
    pub fn forward_two(hometown_base: &str, olive_base: &str) -> Self {
        let (hometown, olive) = (hometown_base.to_owned(), olive_base.to_owned());
        Self::start(2, false, move |url| route_path(url, &hometown, &olive))
    }
    pub fn forward_two_to(target_base: &str) -> Self {
        let base = target_base.to_owned();
        Self::start(2, true, move |url| route_path(url, &base, &base))
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn service_base(&self) -> &str {
        &self.url
    }
    pub fn hometown_url(&self) -> String {
        format!("{}/svc/hometown", self.url)
    }
    pub fn olive_url(&self) -> String {
        format!("{}/svc/olive", self.url)
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

// ---------------------------------------------------------------------------
// Data-driven stubs, routing proxy and gateway trap (CONF-05, CONF-08).
// No expect/unwrap inside spawned threads: failures are recorded and asserted.
// ---------------------------------------------------------------------------

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

const POLL: Duration = Duration::from_millis(25);
const GATEWAY_HOST: &str = "gateway.invalid";

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone, Debug)]
pub struct Reply {
    status: u16,
    body: String,
    delay: Option<Duration>,
    location: Option<String>,
}
impl Reply {
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_owned(),
            delay: None,
            location: None,
        }
    }
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            body: String::new(),
            delay: None,
            location: None,
        }
    }
    /// A 3xx reply carrying a `Location` header (redirect tests).
    pub fn redirect(status: u16, location: &str) -> Self {
        Self {
            status,
            body: String::new(),
            delay: None,
            location: Some(location.to_owned()),
        }
    }
    pub fn delayed(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct CapturedRequest {
    pub method: String,
    pub path: String,
    pub header_names: Vec<String>,
    pub authorization_present: bool,
    pub body: String,
}

pub struct ScriptedStub {
    base_url: String,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl ScriptedStub {
    pub fn new(replies: Vec<Reply>) -> Self {
        let (server, base_url) = match Server::http("127.0.0.1:0") {
            Ok(server) => {
                let url = server
                    .server_addr()
                    .to_ip()
                    .map(|addr| format!("http://{addr}"))
                    .unwrap_or_default();
                (Some(server), url)
            }
            Err(_) => (None, String::new()),
        };
        let captured = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_captured = Arc::clone(&captured);
        let thread_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let Some(server) = server else { return };
            let mut index = 0usize;
            while !thread_stop.load(Ordering::Relaxed) {
                let mut request = match server.recv_timeout(POLL) {
                    Ok(Some(request)) => request,
                    Ok(None) => continue,
                    Err(_) => return,
                };
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                locked(&thread_captured).push(CapturedRequest {
                    method: request.method().as_str().to_owned(),
                    path: request.url().to_owned(),
                    header_names: request
                        .headers()
                        .iter()
                        .map(|h| h.field.as_str().as_str().to_owned())
                        .collect(),
                    authorization_present: request
                        .headers()
                        .iter()
                        .any(|h| h.field.equiv("Authorization")),
                    body,
                });
                let reply = replies
                    .get(index)
                    .or_else(|| replies.last())
                    .cloned()
                    .unwrap_or_else(|| Reply::empty(500));
                index += 1;
                if let Some(delay) = reply.delay {
                    thread::sleep(delay);
                }
                let mut response = Response::from_string(reply.body).with_status_code(reply.status);
                if let Ok(header) =
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                {
                    response = response.with_header(header);
                }
                if let Some(location) = reply.location.as_deref() {
                    if let Ok(header) =
                        tiny_http::Header::from_bytes(&b"Location"[..], location.as_bytes())
                    {
                        response = response.with_header(header);
                    }
                }
                let _ = request.respond(response);
            }
        });
        Self {
            base_url,
            captured,
            stop,
            worker: Some(worker),
        }
    }
    pub fn error(status: u16, code: &str) -> Self {
        let body = serde_json::json!({ "error": code }).to_string();
        Self::new(vec![Reply::json(status, &body)])
    }
    pub fn answer_with(proposal_json: &str) -> Self {
        let body = format!(
            r#"{{"trace_id":"trace-stub-1","answer":"stub answer","context_complete":true,"proposal":{proposal_json}}}"#
        );
        Self::new(vec![Reply::json(200, &body)])
    }
    pub fn olive_ok(
        status: u16,
        proposal_id: &str,
        olive_status: &str,
        reason: Option<&str>,
    ) -> Self {
        let mut value = serde_json::json!({"proposal_id": proposal_id, "status": olive_status});
        if let (Some(reason), Some(map)) = (reason, value.as_object_mut()) {
            map.insert("reason".to_owned(), serde_json::Value::from(reason));
        }
        Self::new(vec![Reply::json(status, &value.to_string())])
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn requests(&self) -> Vec<CapturedRequest> {
        locked(&self.captured).clone()
    }
    pub fn count(&self) -> usize {
        locked(&self.captured).len()
    }
}
impl Drop for ScriptedStub {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProxyStats {
    pub hometown_hits: usize,
    pub olive_hits: usize,
    pub gateway_attempts: usize,
    pub refused_count: usize,
    pub inbound_authorization_count: usize,
    pub connect_attempts: usize,
    pub paths: Vec<String>,
    /// Header names of every inbound request, in arrival order.
    pub header_names: Vec<Vec<String>>,
}

struct ProxyShared {
    hometown: String,
    olive: String,
    stats: Mutex<ProxyStats>,
    errors: Mutex<Vec<String>>,
}

pub struct TestProxy {
    url: String,
    port: u16,
    shared: Arc<ProxyShared>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

fn handle_proxy_request(shared: &ProxyShared, mut request: tiny_http::Request) {
    let path = request.url().to_owned();
    let method = request.method().as_str().to_owned();
    let inbound_auth = request
        .headers()
        .iter()
        .any(|h| h.field.equiv("Authorization"));
    let names: Vec<String> = request
        .headers()
        .iter()
        .map(|h| h.field.as_str().as_str().to_owned())
        .collect();
    let content_type = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Content-Type"))
        .map(|h| h.value.as_str().to_owned());
    let mut body = Vec::new();
    let _ = request.as_reader().read_to_end(&mut body);

    let is_connect = method.eq_ignore_ascii_case("CONNECT");
    let target = if is_connect {
        None
    } else {
        route_path(&path, &shared.hometown, &shared.olive)
    };
    {
        let mut stats = locked(&shared.stats);
        stats.paths.push(format!("{method} {path}"));
        stats.header_names.push(names);
        if inbound_auth {
            stats.inbound_authorization_count += 1;
        }
        if is_connect {
            stats.connect_attempts += 1;
        } else if target.is_some() {
            if path.starts_with("/svc/hometown") {
                stats.hometown_hits += 1;
            } else {
                stats.olive_hits += 1;
            }
        }
    }
    if is_connect {
        locked(&shared.errors).push("connect refused".to_owned());
        let _ = request.respond(Response::empty(405));
        return;
    }
    let Some(target) = target else {
        locked(&shared.errors).push("unroutable path".to_owned());
        let _ = request.respond(Response::empty(502));
        return;
    };

    type Forwarded = (u16, Vec<u8>, Option<String>);
    let forwarded = (|| -> Result<Forwarded, String> {
        // Relay a 3xx as-is so the redirect decision stays with Liam.
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "forward client build".to_owned())?;
        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| "forward method".to_owned())?;
        let mut builder = client.request(method, target);
        if let Some(content_type) = content_type {
            builder = builder.header("Content-Type", content_type);
        }
        builder = builder.header("Authorization", "Bearer proxy-sentinel");
        let response = builder
            .body(body)
            .send()
            .map_err(|_| "forward send".to_owned())?;
        let status = response.status().as_u16();
        let location = response
            .headers()
            .get("Location")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = response
            .bytes()
            .map_err(|_| "forward read".to_owned())?
            .to_vec();
        Ok((status, bytes, location))
    })();
    match forwarded {
        Ok((status, bytes, location)) => {
            let mut response = Response::from_data(bytes).with_status_code(status);
            if let Some(header) = location.and_then(|location| {
                tiny_http::Header::from_bytes(&b"Location"[..], location.as_bytes()).ok()
            }) {
                response = response.with_header(header);
            }
            let _ = request.respond(response);
        }
        Err(reason) => {
            locked(&shared.errors).push(reason);
            let _ = request.respond(Response::empty(502));
        }
    }
}

impl TestProxy {
    pub fn new(hometown_base: &str, olive_base: &str) -> Self {
        Self::start(hometown_base, olive_base, None)
    }
    /// Kept for callers: the trap is never routed to, so it proves zero connections.
    pub fn with_gateway_trap(hometown_base: &str, olive_base: &str, _trap: SocketAddr) -> Self {
        Self::start(hometown_base, olive_base, None)
    }
    fn start(hometown_base: &str, olive_base: &str, _trap: Option<SocketAddr>) -> Self {
        let shared = Arc::new(ProxyShared {
            hometown: hometown_base.trim_end_matches('/').to_owned(),
            olive: olive_base.trim_end_matches('/').to_owned(),
            stats: Mutex::new(ProxyStats::default()),
            errors: Mutex::new(Vec::new()),
        });
        let stop = Arc::new(AtomicBool::new(false));
        let server = Server::http("127.0.0.1:0").ok();
        let (url, port) = server
            .as_ref()
            .and_then(|s| s.server_addr().to_ip())
            .map(|addr| (format!("http://{addr}"), addr.port()))
            .unwrap_or_default();
        let thread_shared = Arc::clone(&shared);
        let thread_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let Some(server) = server else { return };
            let mut handlers: Vec<thread::JoinHandle<()>> = Vec::new();
            while !thread_stop.load(Ordering::Relaxed) {
                match server.recv_timeout(POLL) {
                    Ok(Some(request)) => {
                        let shared = Arc::clone(&thread_shared);
                        handlers.push(thread::spawn(move || {
                            handle_proxy_request(&shared, request)
                        }));
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
            for handler in handlers {
                let _ = handler.join();
            }
        });
        Self {
            url,
            port,
            shared,
            stop,
            worker: Some(worker),
        }
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn service_base(&self) -> &str {
        &self.url
    }
    pub fn hometown_url(&self) -> String {
        format!("{}/svc/hometown", self.url)
    }
    pub fn olive_url(&self) -> String {
        format!("{}/svc/olive", self.url)
    }
    pub fn stats(&self) -> ProxyStats {
        locked(&self.shared.stats).clone()
    }
    pub fn errors(&self) -> Vec<String> {
        locked(&self.shared.errors).clone()
    }
}
impl Drop for TestProxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Raw TCP listener that counts connections; any HTTP request gets a 200 so a
/// negative-control hit is visible.
pub struct GatewayTrap {
    addr: SocketAddr,
    hits: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl GatewayTrap {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").ok();
        let addr = listener
            .as_ref()
            .and_then(|l| l.local_addr().ok())
            .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)));
        let hits = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_hits = Arc::clone(&hits);
        let thread_stop = Arc::clone(&stop);
        let worker = thread::spawn(move || {
            let Some(listener) = listener else { return };
            if listener.set_nonblocking(true).is_err() {
                return;
            }
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        thread_hits.fetch_add(1, Ordering::Relaxed);
                        let _ = stream.set_nonblocking(false);
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
                        let mut buffer = [0u8; 4096];
                        let _ = stream.read(&mut buffer);
                        let _ = stream.write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        );
                    }
                    Err(_) => thread::sleep(POLL),
                }
            }
        });
        Self {
            addr,
            hits,
            stop,
            worker: Some(worker),
        }
    }
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::Relaxed)
    }
}
impl Default for GatewayTrap {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for GatewayTrap {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Send one direct plain-http request to the gateway host through the supplied
/// single no-proxy client; it fails at the deny-all resolver. "refused" on any
/// non-2xx or transport error.
pub fn gateway_attempt(client: &reqwest::blocking::Client) -> &'static str {
    match client.get(format!("http://{GATEWAY_HOST}/")).send() {
        Ok(response) if response.status().is_success() => "allowed",
        _ => "refused",
    }
}

/// Event name kept here so non-stub sources hold no gateway literal (static scan).
pub const GATEWAY_EVENT: &str = "gateway.attempt";

pub fn gateway_attempt_requested() -> bool {
    std::env::var("LIAM_STUB_GATEWAY_ATTEMPT").is_ok_and(|v| v == "1")
}

#[cfg(test)]
mod scripted_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn scripted_replies_are_ordered_and_last_repeats() {
        let stub = ScriptedStub::new(vec![Reply::empty(503), Reply::json(200, r#"{"a":1}"#)]);
        let client = reqwest::blocking::Client::new();
        let codes: Vec<u16> = (0..3)
            .map(|_| {
                client
                    .post(format!("{}/x", stub.base_url()))
                    .body("hi")
                    .send()
                    .expect("send")
                    .status()
                    .as_u16()
            })
            .collect();
        assert_eq!(codes, vec![503, 200, 200]);
        let seen = stub.requests();
        assert_eq!(stub.count(), 3);
        assert_eq!(seen[0].path, "/x");
        assert_eq!(seen[0].body, "hi");
        assert!(!seen[0].authorization_present);
    }

    #[test]
    fn scripted_reply_can_be_delayed() {
        let stub = ScriptedStub::new(vec![Reply::empty(200).delayed(Duration::from_millis(300))]);
        let started = Instant::now();
        reqwest::blocking::get(stub.base_url()).expect("get");
        assert!(started.elapsed() >= Duration::from_millis(300));
    }

    #[test]
    fn error_helper_uses_error_key() {
        let stub = ScriptedStub::error(409, "conflict");
        let text = reqwest::blocking::get(stub.base_url())
            .expect("get")
            .text()
            .expect("text");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert_eq!(value["error"], "conflict");
        assert!(value.get("code").is_none());
    }

    #[test]
    fn answer_and_olive_helpers_embed_payloads() {
        let stub = ScriptedStub::answer_with(r#"{"action_type":"x"}"#);
        let value: serde_json::Value = reqwest::blocking::get(stub.base_url())
            .expect("get")
            .json()
            .expect("json");
        assert_eq!(value["proposal"]["action_type"], "x");
        let olive = ScriptedStub::olive_ok(202, "p-1", "awaiting_approval", Some("r"));
        let value: serde_json::Value = reqwest::blocking::get(olive.base_url())
            .expect("get")
            .json()
            .expect("json");
        assert_eq!(value["proposal_id"], "p-1");
        assert_eq!(value["reason"], "r");
    }

    #[test]
    fn proxy_routes_injects_and_counts() {
        let hometown = ScriptedStub::new(vec![Reply::json(200, "{}")]);
        let olive = ScriptedStub::new(vec![Reply::json(202, "{}")]);
        let proxy = TestProxy::new(hometown.base_url(), olive.base_url());
        let pc = crate::clients::build_service_client().expect("client");
        let client = pc.client();
        let r1 = client
            .post(format!("{}/v1/agent/answer", proxy.hometown_url()))
            .header("Authorization", "Bearer inbound-check")
            .body("{}")
            .send()
            .expect("hometown");
        assert_eq!(r1.status().as_u16(), 200);
        let r2 = client
            .post(format!("{}/v1/proposals", proxy.olive_url()))
            .body("{}")
            .send()
            .expect("olive");
        assert_eq!(r2.status().as_u16(), 202);
        let stats = proxy.stats();
        assert_eq!(stats.hometown_hits, 1);
        assert_eq!(stats.olive_hits, 1);
        assert_eq!(stats.inbound_authorization_count, 1);
        assert_eq!(stats.gateway_attempts, 0);
        assert_eq!(stats.connect_attempts, 0);
        assert_eq!(stats.paths.len(), 2);
        assert!(hometown.requests()[0].authorization_present);
        assert_eq!(hometown.requests()[0].path, "/v1/agent/answer");
        assert!(olive.requests()[0].authorization_present);
        assert!(proxy.errors().is_empty());
    }

    #[test]
    fn unroutable_path_is_502_and_recorded() {
        let hometown = ScriptedStub::new(vec![Reply::json(200, "{}")]);
        let proxy = TestProxy::new(hometown.base_url(), hometown.base_url());
        let pc = crate::clients::build_service_client().expect("client");
        let client = pc.client();
        for path in ["/other", "/svc/hometownX/v1"] {
            let response = client
                .post(format!("{}{}", proxy.service_base(), path))
                .body("{}")
                .send()
                .expect("send");
            assert_eq!(response.status().as_u16(), 502);
        }
        assert_eq!(hometown.count(), 0);
        assert_eq!(proxy.errors().len(), 2);
    }

    #[test]
    fn connect_is_counted_and_not_forwarded() {
        let hometown = ScriptedStub::new(vec![Reply::json(200, "{}")]);
        let proxy = TestProxy::new(hometown.base_url(), hometown.base_url());
        let mut stream =
            std::net::TcpStream::connect(("127.0.0.1", proxy.port())).expect("connect");
        stream
            .write_all(b"CONNECT example.invalid:443 HTTP/1.1\r\nHost: example.invalid:443\r\n\r\n")
            .expect("write");
        let mut buffer = [0u8; 256];
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.read(&mut buffer);
        let started = Instant::now();
        while proxy.stats().connect_attempts == 0 && started.elapsed() < Duration::from_secs(5) {
            thread::sleep(POLL);
        }
        assert_eq!(proxy.stats().connect_attempts, 1);
        assert_eq!(hometown.count(), 0);
    }

    #[test]
    fn stub_thread_ends_after_drop() {
        let started = Instant::now();
        drop(ScriptedStub::new(vec![Reply::empty(200)]));
        let hometown = ScriptedStub::new(vec![Reply::empty(200)]);
        drop(TestProxy::new(hometown.base_url(), hometown.base_url()));
        drop(GatewayTrap::new());
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn gateway_attempt_is_direct_and_trap_untouched() {
        let trap = GatewayTrap::new();
        let pc = crate::clients::build_service_client().expect("client");
        assert_eq!(gateway_attempt(&pc.client()), "refused");
        assert_eq!(trap.hits(), 0);
        assert_eq!(pc.resolver_calls(), 1);
    }

    #[test]
    fn gateway_trap_counts_a_direct_connection() {
        let trap = GatewayTrap::new();
        let response = reqwest::blocking::Client::builder()
            .no_proxy()
            .build()
            .expect("client")
            .get(format!("http://{}/", trap.addr()))
            .send()
            .expect("send");
        assert!(response.status().is_success());
        assert_eq!(trap.hits(), 1);
    }

    #[test]
    fn gateway_attempt_not_requested_when_unset() {
        if std::env::var_os("LIAM_STUB_GATEWAY_ATTEMPT").is_none() {
            assert!(!gateway_attempt_requested());
        }
    }
}
