//! Catalog actions and a read-only validator for a small JSON-Schema subset.
//!
//! Pure module: no network, no coercion, no mutation of the proposal.
//! "integer" means `Number::is_i64()` or `is_u64()`; this is stricter than
//! JSON Schema 2020-12, where 5.0 is an integer. `maxLength` counts Unicode
//! scalar values. An object schema with no `properties` accepts any nested
//! content without recursing into it, unless `additionalProperties` is false,
//! in which case it accepts only the empty object.
//!
//! Fail closed on drift: a catalog or schema keyword outside this subset is a
//! parse error, so an unsupported catalog stops the run rather than having its
//! constraints silently ignored.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// The only `action_id_format` this validator implements (`action_id_format_ok`).
pub const ACTION_ID_FORMAT: &str = "^[a-z0-9_.]+@[0-9]+$";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub version: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub action_id_format: Option<String>,
    pub actions: Vec<CatalogAction>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogAction {
    pub action_type: String,
    pub parameters_schema: Schema,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub properties: Option<BTreeMap<String, Schema>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(rename = "additionalProperties", default = "default_true")]
    pub additional_properties: bool,
    #[serde(rename = "maxLength", default)]
    pub max_length: Option<u64>,
}

fn default_true() -> bool {
    true
}

impl Catalog {
    pub fn action(&self, id: &str) -> Option<&CatalogAction> {
        self.actions.iter().find(|a| a.action_type == id)
    }
    /// True only when the catalog declares the id format this code checks.
    pub fn action_id_format_supported(&self) -> bool {
        self.action_id_format.as_deref() == Some(ACTION_ID_FORMAT)
    }
}

/// Hand-written check of ^[a-z0-9_.]+@[0-9]+$.
pub fn action_id_format_ok(id: &str) -> bool {
    let Some((name, ver)) = id.split_once('@') else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.')
        && !ver.is_empty()
        && ver.bytes().all(|b| b.is_ascii_digit())
}

pub fn validate_parameters(schema: &Schema, value: &Value) -> Result<(), &'static str> {
    match schema.kind.as_str() {
        "object" => {
            let obj = value.as_object().ok_or("schema")?;
            if schema.required.iter().any(|k| !obj.contains_key(k)) {
                return Err("schema");
            }
            let Some(props) = &schema.properties else {
                if !schema.additional_properties && !obj.is_empty() {
                    return Err("schema");
                }
                return Ok(());
            };
            for (key, val) in obj {
                match props.get(key) {
                    Some(sub) => validate_parameters(sub, val)?,
                    None if !schema.additional_properties => return Err("schema"),
                    None => {}
                }
            }
            Ok(())
        }
        "string" => {
            let s = value.as_str().ok_or("schema")?;
            match schema.max_length {
                Some(max) if (s.chars().count() as u64) > max => Err("schema"),
                _ => Ok(()),
            }
        }
        "integer" => match value {
            Value::Number(n) if n.is_i64() || n.is_u64() => Ok(()),
            _ => Err("schema"),
        },
        "number" => value.is_number().then_some(()).ok_or("schema"),
        "boolean" => value.is_boolean().then_some(()).ok_or("schema"),
        _ => Err("schema"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn catalog() -> Catalog {
        serde_json::from_str(include_str!("../contracts/catalog-0.4.json")).unwrap()
    }

    fn check(id: &str, v: Value) -> Result<(), &'static str> {
        validate_parameters(&catalog().action(id).unwrap().parameters_schema, &v)
    }

    #[test]
    fn integer_is_not_coerced() {
        let id = "crm.follow_up@1";
        assert!(check(id, json!({"days": 5})).is_ok());
        assert!(check(id, json!({"days": "5"})).is_err());
        assert!(check(id, json!({"days": 5.0})).is_err());
        assert!(check(id, json!({"days": 5, "x": 1})).is_err());
        assert!(check(id, json!({})).is_err());
    }

    #[test]
    fn max_length_counts_scalars() {
        let id = "crm.case_note@1";
        let ok = "é".repeat(2000);
        assert!(check(id, json!({"note": ok})).is_ok());
        let long = "é".repeat(2001);
        assert!(check(id, json!({"note": long})).is_err());
    }

    #[test]
    fn free_form_details_accept_nested_content() {
        let v = json!({"details": {"a": [1.5, {"b": "ü"}], "z": null, "n": {"x": 0.25}}});
        assert!(check("crm.case_update@1", v).is_ok());
    }

    #[test]
    fn action_id_format() {
        assert!(action_id_format_ok("crm.case_note@1"));
        for bad in ["CRM.x@1", "a@", "a@b", "a b@1", "@1"] {
            assert!(!action_id_format_ok(bad), "{bad}");
        }
    }

    #[test]
    fn unknown_action_is_none() {
        assert!(catalog().action("crm.nope@1").is_none());
    }

    #[test]
    fn pinned_catalog_declares_the_supported_id_format() {
        assert!(catalog().action_id_format_supported());
        let other: Catalog =
            serde_json::from_str(r#"{"version":"0.4","action_id_format":"^.+$","actions":[]}"#)
                .unwrap();
        assert!(!other.action_id_format_supported());
        let missing: Catalog = serde_json::from_str(r#"{"version":"0.4","actions":[]}"#).unwrap();
        assert!(!missing.action_id_format_supported());
    }

    #[test]
    fn unsupported_keywords_fail_to_parse() {
        for schema in [
            r#"{"type":"string","enum":["a"]}"#,
            r#"{"type":"string","pattern":"^a$"}"#,
            r#"{"type":"integer","minimum":1}"#,
            r#"{"type":"string","minLength":1}"#,
            r#"{"type":"object","properties":{"x":{"type":"string","const":"a"}}}"#,
        ] {
            let text = format!(
                r#"{{"version":"0.4","action_id_format":"{}","actions":[{{"action_type":"a.b@1","parameters_schema":{schema}}}]}}"#,
                ACTION_ID_FORMAT
            );
            assert!(serde_json::from_str::<Catalog>(&text).is_err(), "{schema}");
        }
        for extra in [
            r#"{"version":"0.4","actions":[],"extra":1}"#,
            r#"{"version":"0.4","actions":[{"action_type":"a.b@1","parameters_schema":{"type":"object"},"x":1}]}"#,
        ] {
            assert!(serde_json::from_str::<Catalog>(extra).is_err(), "{extra}");
        }
    }

    #[test]
    fn closed_object_without_properties_accepts_only_empty() {
        let schema: Schema =
            serde_json::from_str(r#"{"type":"object","additionalProperties":false}"#).unwrap();
        assert!(validate_parameters(&schema, &json!({})).is_ok());
        assert!(validate_parameters(&schema, &json!({"a": 1})).is_err());
        let open: Schema = serde_json::from_str(r#"{"type":"object"}"#).unwrap();
        assert!(validate_parameters(&open, &json!({"a": 1})).is_ok());
    }
}
