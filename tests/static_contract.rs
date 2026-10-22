//! Static TC-NET-03 / TC-ANS-04 / TC-ACT-05 / TC-SEC-01 checks over non-stub `src/**/*.rs`.
//! Runs under default features; the scanner is comment- and string-aware.

use std::{
    fs,
    path::{Path, PathBuf},
};

struct Scanned {
    code: String,
    strings: Vec<String>,
}

fn scan(source: &str) -> Scanned {
    let c: Vec<char> = source.chars().collect();
    let mut code = String::new();
    let mut strings = Vec::new();
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        let next = c.get(i + 1).copied();
        if ch == '/' && next == Some('/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && next == Some('*') {
            let mut depth = 1;
            i += 2;
            while i < c.len() && depth > 0 {
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            code.push(' ');
        } else if ch == 'r' && matches!(next, Some('"') | Some('#')) && is_raw_start(&c, i) {
            let mut j = i + 1;
            let mut hashes = 0;
            while c[j] == '#' {
                hashes += 1;
                j += 1;
            }
            j += 1; // opening quote
            let mut s = String::new();
            while j < c.len() {
                if c[j] == '"' && (0..hashes).all(|k| c.get(j + 1 + k) == Some(&'#')) {
                    j += 1 + hashes;
                    break;
                }
                s.push(c[j]);
                j += 1;
            }
            strings.push(s);
            code.push_str("\"\"");
            i = j;
        } else if ch == '"' {
            let mut s = String::new();
            i += 1;
            while i < c.len() && c[i] != '"' {
                if c[i] == '\\' && i + 1 < c.len() {
                    s.push(c[i + 1]);
                    i += 2;
                } else {
                    s.push(c[i]);
                    i += 1;
                }
            }
            i += 1;
            strings.push(s);
            code.push_str("\"\"");
        } else if ch == '\'' {
            // char literal ('x', '\n', '\'') vs lifetime ('a)
            if next == Some('\\') {
                let mut j = i + 2;
                while j < c.len() && c[j] != '\'' {
                    j += 1;
                }
                code.push_str("' '");
                i = j + 1;
            } else if c.get(i + 2) == Some(&'\'') {
                code.push_str("' '");
                i += 3;
            } else {
                code.push(ch);
                i += 1;
            }
        } else {
            code.push(ch);
            i += 1;
        }
    }
    Scanned { code, strings }
}

fn is_raw_start(c: &[char], i: usize) -> bool {
    if i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_') {
        return false;
    }
    let mut j = i + 1;
    while c.get(j) == Some(&'#') {
        j += 1;
    }
    c.get(j) == Some(&'"')
}

const FORBIDDEN_CODE: &[&str] = &[
    "std::net",
    "ToSocketAddrs",
    "lookup_host",
    "TcpStream",
    "TcpListener",
    "UdpSocket",
    "getaddrinfo",
    "WSAStartup",
    "Win32::Networking",
    "Command::new",
    "std::process::Command",
    "Proxy::",
];

const CLIENT_CODE: &[&str] = &["ClientBuilder", "Client::new", "Client::builder"];

const FORBIDDEN_STRINGS: &[&str] = &[
    "gateway.",
    "gateway.invalid",
    "/v1/chat/completions",
    "/v1/messages",
    "/v1/completions",
];

/// Quoted env-var names Liam must never read (D-06, D-10); matched exactly.
const FORBIDDEN_ENV_LITERALS: &[&str] = &[
    "HTTPS_PROXY",
    "HTTP_PROXY",
    "NO_PROXY",
    "ALL_PROXY",
    "NONO_SERVICE_GATEWAY_URL",
    "https_proxy",
    "http_proxy",
    "no_proxy",
    "all_proxy",
];

fn env_literal_violations(strings: &[String]) -> Vec<String> {
    strings
        .iter()
        .filter(|s| FORBIDDEN_ENV_LITERALS.contains(&s.as_str()))
        .map(|s| format!("env literal {s:?}"))
        .collect()
}

fn count_token(code: &str, tok: &str) -> usize {
    code.match_indices(tok)
        .filter(|(i, _)| {
            code[..*i]
                .chars()
                .next_back()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        })
        .count()
}

fn code_violations(code: &str, allow_client: bool) -> Vec<String> {
    let mut v: Vec<String> = FORBIDDEN_CODE
        .iter()
        .filter(|t| count_token(code, t) > 0)
        .map(|t| t.to_string())
        .collect();
    if !allow_client {
        v.extend(
            CLIENT_CODE
                .iter()
                .filter(|t| count_token(code, t) > 0)
                .map(|t| t.to_string()),
        );
    }
    v
}

fn string_violations(strings: &[String]) -> Vec<String> {
    let mut v = Vec::new();
    for s in strings {
        for t in FORBIDDEN_STRINGS {
            if s.contains(t) {
                v.push(format!("{t} in {s:?}"));
            }
        }
        if s.to_ascii_lowercase().starts_with("authorization") {
            v.push(format!("Authorization literal {s:?}"));
        }
    }
    v
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn non_stub_sources() -> Vec<(PathBuf, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let stubs = src.join("stubs.rs");
    let mut files = Vec::new();
    collect(&src, &mut files);
    files.retain(|p| *p != stubs);
    assert!(files.len() >= 5, "source walk found too few files");
    files
        .into_iter()
        .map(|p| {
            let t = fs::read_to_string(&p).expect("read source");
            (p, t)
        })
        .collect()
}

fn is_client_mod(p: &Path) -> bool {
    p.ends_with(Path::new("clients").join("mod.rs"))
}

#[test]
fn tc_net_03_no_socket_or_resolver_apis() {
    for (path, text) in non_stub_sources() {
        let s = scan(&text);
        let v = code_violations(&s.code, is_client_mod(&path));
        assert!(v.is_empty(), "{}: forbidden code {:?}", path.display(), v);
    }
}

#[test]
fn tc_net_03_single_proxy_client_constructor() {
    let mut builders = 0;
    for (path, text) in non_stub_sources() {
        let s = scan(&text);
        let n = count_token(&s.code, "Client::builder");
        if is_client_mod(&path) {
            assert_eq!(n, 1, "exactly one Client::builder expected");
            for needle in [
                "no_proxy()",
                "dns_resolver(",
                "redirect(reqwest::redirect::Policy::none())",
            ] {
                assert!(s.code.contains(needle), "clients/mod.rs missing {needle}");
            }
            for banned in ["Proxy::", ".proxy("] {
                assert!(!s.code.contains(banned), "clients/mod.rs contains {banned}");
            }
        } else {
            assert_eq!(n, 0, "{}: second client constructor", path.display());
        }
        builders += n;
    }
    assert_eq!(builders, 1);
}

#[test]
fn tc_net_03_no_ambient_proxy_env_or_gateway_url_reads() {
    for (path, text) in non_stub_sources() {
        let v = env_literal_violations(&scan(&text).strings);
        assert!(v.is_empty(), "{}: {:?}", path.display(), v);
    }
}

#[test]
fn liam_toml_has_no_base_url() {
    let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("liam.toml"))
        .expect("read liam.toml");
    assert!(!text.contains("base_url"));
}

#[test]
fn tc_net_03_negative_control_env_literals() {
    for t in FORBIDDEN_ENV_LITERALS {
        let src = format!("fn f() {{ let _ = std::env::var(\"{t}\"); }}");
        assert!(
            !env_literal_violations(&scan(&src).strings).is_empty(),
            "missed {t}"
        );
    }
    let flagged = scan("fn f() { let _ = env::var(\"NO_PROXY\"); }");
    assert!(!env_literal_violations(&flagged.strings).is_empty());
    let exempt = scan("fn f() { Client::builder().no_proxy(); }");
    assert!(env_literal_violations(&exempt.strings).is_empty());
    assert!(code_violations(&exempt.code, true).is_empty());
}

#[test]
fn tc_ans_04_no_gateway_reference() {
    for (path, text) in non_stub_sources() {
        let v = string_violations(&scan(&text).strings);
        assert!(v.is_empty(), "{}: {:?}", path.display(), v);
    }
    assert!(string_violations(&["direct_gateway".to_string()]).is_empty());
}

#[test]
fn tc_act_05_no_process_spawn() {
    for (path, text) in non_stub_sources() {
        let code = scan(&text).code;
        assert!(
            !code.contains("Command::new") && !code.contains("std::process::Command"),
            "{}: process spawn",
            path.display()
        );
    }
}

#[test]
fn tc_sec_01_no_authorization_literal() {
    for (path, text) in non_stub_sources() {
        let s = scan(&text);
        assert!(
            !s.strings
                .iter()
                .any(|x| x.to_ascii_lowercase().starts_with("authorization")),
            "{}: Authorization literal",
            path.display()
        );
    }
}

#[test]
fn tc_net_03_negative_control_code_tokens() {
    for t in FORBIDDEN_CODE.iter().chain(CLIENT_CODE) {
        let src = format!("fn f() {{ let _ = {t}; }}");
        let v = code_violations(&scan(&src).code, false);
        assert!(v.iter().any(|x| x == t), "scanner missed {t}");
    }
    assert!(code_violations(&scan("fn f() { Client::builder(); }").code, true).is_empty());
}

#[test]
fn tc_net_03_negative_control_string_literals() {
    for t in FORBIDDEN_STRINGS {
        let src = format!("fn f() {{ let _ = \"https://{t}x\"; }}");
        assert!(
            !string_violations(&scan(&src).strings).is_empty(),
            "missed {t}"
        );
    }
    let raw = "fn f() { let _ = r#\"/v1/messages\"#; }";
    assert!(!string_violations(&scan(raw).strings).is_empty());
    let auth = "fn f() { let _ = \"Authorization\"; }";
    assert!(!string_violations(&scan(auth).strings).is_empty());
}

#[test]
fn tc_net_03_negative_control_no_false_positives() {
    let src = "// gateway std::net\n/* TcpStream /* nested */ lookup_host */\n\
               fn f() { let _ = \"https://example\"; let _ = \"direct_gateway\"; let _ = '\"'; }";
    let s = scan(src);
    assert!(code_violations(&s.code, false).is_empty(), "{}", s.code);
    assert!(string_violations(&s.strings).is_empty());
    assert!(s.strings.contains(&"https://example".to_string()));
}

#[test]
fn tc_net_03_negative_control_strings_hide_code_tokens() {
    let s = scan("fn f() { let _ = \"std::net\"; }");
    assert!(code_violations(&s.code, false).is_empty());
}
