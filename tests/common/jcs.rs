//! Test-only RFC 8785 (JCS) canonicaliser. Liam never computes JCS at runtime (D-02).
//! Number digits come from Rust's shortest round-trip `{:e}` output (same digits as ryu),
//! re-laid out per ECMAScript Number::toString.
#![allow(dead_code)]

use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn canonicalise(v: &Value) -> String {
    let mut out = String::new();
    write_value(v, &mut out);
    out
}

pub fn jcs_sha256_hex(v: &Value) -> String {
    let digest = Sha256::digest(canonicalise(v).as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn write_value(v: &Value, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => out.push_str(&number(n.as_f64().unwrap_or(0.0))),
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            out.push('{');
            for (i, (key, value)) in entries.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                write_value(value, out);
            }
            out.push('}');
        }
    }
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn number(f: f64) -> String {
    if f == 0.0 {
        return "0".to_owned();
    }
    let sci = format!("{:e}", f.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let exp: i32 = exp.parse().unwrap_or(0);
    let k = digits.len() as i32;
    let n = exp + 1;
    let mut s = String::new();
    if f < 0.0 {
        s.push('-');
    }
    if k <= n && n <= 21 {
        s.push_str(&digits);
        s.push_str(&"0".repeat((n - k) as usize));
    } else if 0 < n && n <= 21 {
        s.push_str(&digits[..n as usize]);
        s.push('.');
        s.push_str(&digits[n as usize..]);
    } else if -6 < n && n <= 0 {
        s.push_str("0.");
        s.push_str(&"0".repeat((-n) as usize));
        s.push_str(&digits);
    } else {
        let e = n - 1;
        s.push_str(&digits[..1]);
        if k > 1 {
            s.push('.');
            s.push_str(&digits[1..]);
        }
        s.push('e');
        s.push(if e < 0 { '-' } else { '+' });
        s.push_str(&e.abs().to_string());
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canon(text: &str) -> String {
        canonicalise(&serde_json::from_str(text).unwrap())
    }

    fn float(text: &str) -> Value {
        // serde_json's default float parsing is not exactly rounded; use std.
        Value::from(text.parse::<f64>().unwrap())
    }

    #[test]
    fn jcs_appendix_b_numbers() {
        let cases = [
            ("333333333.33333329", "333333333.3333333"),
            ("1E30", "1e+30"),
            ("4.50", "4.5"),
            ("2e-3", "0.002"),
            ("0.000001", "0.000001"),
            ("1e-7", "1e-7"),
            ("-0", "0"),
            ("1e21", "1e+21"),
            ("999999999999999900000", "999999999999999900000"),
        ];
        for (input, expected) in cases {
            assert_eq!(canonicalise(&float(input)), expected, "input {input}");
        }
    }

    #[test]
    fn jcs_rfc_section_3_2_3_example() {
        let input = serde_json::json!({
            "numbers": [
                float("333333333.33333329"),
                float("1E30"),
                float("4.50"),
                float("2e-3"),
                float("0.000000000000000000000000001")
            ],
            "string": "\u{20ac}$\u{0F}\nA'B\"\\\\\"/",
            "literals": [null, true, false]
        });
        let expected = r#"{"literals":[null,true,false],"numbers":[333333333.3333333,1e+30,4.5,0.002,1e-27],"string":"€$\u000f\nA'B\"\\\\\"/"}"#;
        assert_eq!(canonicalise(&input), expected);
    }

    #[test]
    fn jcs_keys_sort_by_utf16_code_units() {
        // U+10000 is D800 DC00 in UTF-16, which sorts before U+FFFD; UTF-8 byte order would not.
        let out = canon("{\"\u{FFFD}\":1,\"\u{10000}\":2}");
        assert_eq!(out, "{\"\u{10000}\":2,\"\u{FFFD}\":1}");
        let out = canon(r#"{"€":"E","\r":"C","דּ":"H","1":"1","😀":"M","\u0080":"X","ö":"L"}"#);
        let expected_order = [
            "\\r",
            "1",
            "\u{80}",
            "\u{f6}",
            "\u{20ac}",
            "\u{1F600}",
            "\u{fb33}",
        ];
        let positions: Vec<usize> = expected_order
            .iter()
            .map(|k| out.find(&format!("\"{k}\":")).unwrap())
            .collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]), "{out}");
    }

    #[test]
    fn jcs_string_escaping_is_minimal() {
        assert_eq!(
            canon(r#""\b\t\n\f\r\u0001\u001f\"\\\/é""#),
            "\"\\b\\t\\n\\f\\r\\u0001\\u001f\\\"\\\\/é\""
        );
    }
}
