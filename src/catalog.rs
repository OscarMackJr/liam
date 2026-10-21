//! Catalog actions and a read-only validator for a small JSON-Schema subset.
//!
//! Pure module: no network, no coercion, no mutation of the proposal.
//! "integer" means `Number::is_i64()` or `is_u64()`; this is stricter than
//! JSON Schema 2020-12, where 5.0 is an integer. `maxLength` counts Unicode
//! scalar values. An object schema with no `properties` accepts any nested
//! content without recursing into it.

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub version: String,
    #[serde(default)]
    pub action_id_format: Option<String>,
    pub actions: Vec<CatalogAction>,
}

#[derive(Debug, Deserialize)]
pub struct CatalogAction {
    pub action_type: String,
    pub parameters_schema: Schema,
}

#[derive(Debug, Deserialize)]
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
}
