use crate::catalog::{action_id_format_ok, validate_parameters, Catalog};
use crate::clients::{
    hometown::{AnswerRequest, HometownClient, HometownError},
    olive::{OliveClient, OliveError, ProposalRequest, ProposalResponse},
};
use serde::Deserialize;
use serde_json::{value::RawValue, Value};
use std::time::Instant;

#[derive(Debug)]
pub enum TaskError {
    Hometown(HometownError),
    Catalog,
    /// The proposal passed validation (`proposal.validated`) and olive failed.
    Olive {
        answer: String,
        context_complete: bool,
        trace_id: String,
        latency_ms: u64,
        error: OliveError,
    },
}
pub struct TaskResult {
    pub answer: String,
    pub context_complete: bool,
    pub trace_id: String,
    /// Content-free `answer.received` fields (design §3.3 step 2).
    pub has_proposal: bool,
    pub latency_ms: u64,
    /// The proposal passed the §7.3 checks (`proposal.validated`).
    pub validated: bool,
    pub proposal: Option<(String, String)>,
    pub submitted: Option<ProposalResponse>,
    pub suppressed: Option<&'static str>,
    pub limit_suppressed: usize,
}

/// Read-only strict copy used only for validation (D-05): exactly four keys.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FourKeys {
    action_type: String,
    party_ref: String,
    parameters: Value,
    #[allow(dead_code)]
    justification: String,
}
/// Raw copy used for the olive body, so forwarded bytes equal hometown's (D-01).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFour {
    action_type: Box<RawValue>,
    party_ref: Box<RawValue>,
    parameters: Box<RawValue>,
    justification: Box<RawValue>,
}

fn validate(catalog: &Catalog, raw: &str, party_ref: &str) -> Result<(), &'static str> {
    let strict: FourKeys = serde_json::from_str(raw).map_err(|_| "schema")?;
    if strict.party_ref != party_ref {
        return Err("schema");
    }
    let action = catalog
        .action(&strict.action_type)
        .ok_or("not_in_catalog")?;
    if !action_id_format_ok(&strict.action_type) {
        return Err("schema");
    }
    validate_parameters(&action.parameters_schema, &strict.parameters)
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    hometown: &dyn HometownClient,
    olive: &dyn OliveClient,
    catalog_json: &str,
    catalog_version: &str,
    party_ref: &str,
    question: &str,
    purpose: &str,
    run_id: &str,
    session_id: &str,
) -> Result<TaskResult, TaskError> {
    let catalog: Catalog = serde_json::from_str(catalog_json).map_err(|_| TaskError::Catalog)?;
    if catalog.version != catalog_version || !catalog.action_id_format_supported() {
        return Err(TaskError::Catalog);
    }
    let started = Instant::now();
    let answer = hometown
        .answer(&AnswerRequest {
            party_ref,
            question,
            purpose,
            run_id,
            nono_session_id: session_id,
            step_role: "answer",
        })
        .map_err(TaskError::Hometown)?;
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = |proposal: Option<(String, String)>,
                  submitted: Option<ProposalResponse>,
                  suppressed: Option<&'static str>,
                  limit_suppressed: usize| TaskResult {
        answer: answer.answer.clone(),
        context_complete: answer.context_complete,
        trace_id: answer.trace_id.clone(),
        has_proposal: answer.proposal.is_some(),
        latency_ms,
        validated: submitted.is_some(),
        proposal,
        submitted,
        suppressed,
        limit_suppressed,
    };
    let Some(proposal) = answer.proposal.as_deref() else {
        return Ok(result(None, None, None, 0));
    };
    // Assumption A3: a non-conforming array shape uses element 0 and counts the rest.
    let mut limit_suppressed = 0;
    let first: Box<RawValue>;
    let raw: &RawValue = if proposal.get().trim_start().starts_with('[') {
        let items: Vec<Box<RawValue>> = match serde_json::from_str(proposal.get()) {
            Ok(items) => items,
            Err(_) => return Ok(result(None, None, Some("schema"), 0)),
        };
        limit_suppressed = items.len().saturating_sub(1);
        match items.into_iter().next() {
            Some(item) => {
                first = item;
                &first
            }
            None => return Ok(result(None, None, Some("schema"), 0)),
        }
    } else {
        proposal
    };
    if let Err(reason) = validate(&catalog, raw.get(), party_ref) {
        return Ok(result(None, None, Some(reason), limit_suppressed));
    }
    let fields: RawFour = match serde_json::from_str(raw.get()) {
        Ok(fields) => fields,
        Err(_) => return Ok(result(None, None, Some("schema"), limit_suppressed)),
    };
    let response = olive
        .submit(&ProposalRequest {
            action_type: &fields.action_type,
            party_ref: &fields.party_ref,
            parameters: &fields.parameters,
            justification: &fields.justification,
            run_id,
            trace_ref: &answer.trace_id,
        })
        .map_err(|error| TaskError::Olive {
            answer: answer.answer.clone(),
            context_complete: answer.context_complete,
            trace_id: answer.trace_id.clone(),
            latency_ms,
            error,
        })?;
    Ok(result(
        Some((response.proposal_id.clone(), response.status.clone())),
        Some(response),
        None,
        limit_suppressed,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hometown::AnswerResponse;
    use std::cell::RefCell;

    const CATALOG: &str = include_str!("../contracts/catalog-0.4.json");
    const RUN: &str = "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b";

    struct Hometown {
        response: AnswerResponse,
    }
    impl HometownClient for Hometown {
        fn answer(&self, _: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError> {
            Ok(self.response.clone())
        }
    }
    struct Olive {
        bodies: RefCell<Vec<String>>,
        fail: bool,
    }
    impl Olive {
        fn new(fail: bool) -> Self {
            Self {
                bodies: RefCell::new(vec![]),
                fail,
            }
        }
    }
    impl OliveClient for Olive {
        fn submit(&self, request: &ProposalRequest<'_>) -> Result<ProposalResponse, OliveError> {
            assert_eq!(request.trace_ref, "trace-1");
            self.bodies
                .borrow_mut()
                .push(serde_json::to_string(request).unwrap());
            if self.fail {
                return Err(OliveError::Unavailable);
            }
            Ok(ProposalResponse {
                proposal_id: "proposal-1".into(),
                status: "awaiting_approval".into(),
                reason: None,
                http_status: 202,
            })
        }
    }

    fn hometown(proposal: Option<&str>) -> Hometown {
        Hometown {
            response: AnswerResponse {
                trace_id: "trace-1".into(),
                answer: "display only".into(),
                context_complete: true,
                proposal: proposal.map(|p| RawValue::from_string(p.to_string()).unwrap()),
            },
        }
    }
    fn go(h: &Hometown, o: &Olive) -> Result<TaskResult, TaskError> {
        run(
            h,
            o,
            CATALOG,
            "0.4",
            "p",
            "q",
            "servicing",
            RUN,
            "session-1",
        )
    }
    fn suppressed(proposal: &str) -> Option<&'static str> {
        let o = Olive::new(false);
        let r = go(&hometown(Some(proposal)), &o).unwrap();
        assert!(o.bodies.borrow().is_empty());
        r.suppressed
    }

    #[test]
    fn no_proposal_means_nothing_to_submit() {
        let o = Olive::new(false);
        let r = go(&hometown(None), &o).unwrap();
        assert!(r.proposal.is_none() && r.suppressed.is_none());
        assert!(!r.has_proposal && !r.validated);
        assert!(o.bodies.borrow().is_empty());
    }

    #[test]
    fn result_reports_received_and_validated() {
        let one = r#"{"action_type":"crm.case_note@1","party_ref":"p","parameters":{"note":"n"},"justification":"j"}"#;
        let o = Olive::new(false);
        let r = go(&hometown(Some(one)), &o).unwrap();
        assert!(r.has_proposal && r.validated);
        let bad =
            r#"{"action_type":"crm.nope@1","party_ref":"p","parameters":{},"justification":"j"}"#;
        let r = go(&hometown(Some(bad)), &o).unwrap();
        assert!(r.has_proposal && !r.validated);
        assert_eq!(r.suppressed, Some("not_in_catalog"));
    }

    #[test]
    fn submits_valid_proposal_with_verbatim_bytes() {
        let params = r#"{ "note":"caf00e9  1.50" }"#;
        let proposal = format!(
            r#"{{"justification":"why  00e9","parameters":{params},"party_ref":"p","action_type":"crm.case_note@1"}}"#
        );
        let o = Olive::new(false);
        let r = go(&hometown(Some(&proposal)), &o).unwrap();
        assert_eq!(r.proposal.unwrap().1, "awaiting_approval");
        let bodies = o.bodies.borrow();
        assert_eq!(bodies.len(), 1);
        assert!(bodies[0].contains(params));
        assert!(bodies[0].contains(r#""why  00e9""#));
    }

    #[test]
    fn strict_four_key_shape_is_enforced() {
        let ok = r#""action_type":"crm.case_note@1","party_ref":"p","parameters":{"note":"n"},"justification":"j""#;
        assert_eq!(suppressed(&format!("{{{ok},\"extra\":1}}")), Some("schema"));
        assert_eq!(
            suppressed(
                r#"{"action_type":"crm.case_note@1","party_ref":"p","parameters":{"note":"n"}}"#
            ),
            Some("schema")
        );
        assert_eq!(
            suppressed(&format!("{{{ok},\"party_ref\":\"p\"}}")),
            Some("schema")
        );
    }

    #[test]
    fn party_mismatch_and_catalog_checks_suppress() {
        let make = |action: &str, party: &str, params: &str| {
            format!(
                r#"{{"action_type":"{action}","party_ref":"{party}","parameters":{params},"justification":"j"}}"#
            )
        };
        assert_eq!(
            suppressed(&make("crm.case_note@1", "other", r#"{"note":"n"}"#)),
            Some("schema")
        );
        assert_eq!(
            suppressed(&make("crm.nope@1", "p", "{}")),
            Some("not_in_catalog")
        );
        assert_eq!(
            suppressed(&make("crm.follow_up@1", "p", r#"{"days":"5"}"#)),
            Some("schema")
        );
    }

    #[test]
    fn array_shape_uses_first_and_counts_the_rest() {
        let one = r#"{"action_type":"crm.case_note@1","party_ref":"p","parameters":{"note":"n"},"justification":"j"}"#;
        let o = Olive::new(false);
        let r = go(&hometown(Some(&format!("[{one},{one}]"))), &o).unwrap();
        assert_eq!(o.bodies.borrow().len(), 1);
        assert_eq!(r.limit_suppressed, 1);
        assert!(r.proposal.is_some());
    }

    #[test]
    fn unsupported_catalog_is_a_catalog_error_before_hometown() {
        struct NoHometown;
        impl HometownClient for NoHometown {
            fn answer(&self, _: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError> {
                panic!("hometown must not be called with an unsupported catalog");
            }
        }
        let fmt = r#""action_id_format":"^[a-z0-9_.]+@[0-9]+$""#;
        for catalog in [
            // unsupported keyword
            format!(
                r#"{{"version":"0.4",{fmt},"actions":[{{"action_type":"crm.case_note@1","parameters_schema":{{"type":"object","properties":{{"note":{{"type":"string","enum":["a"]}}}}}}}}]}}"#
            ),
            // changed action_id_format
            r#"{"version":"0.4","action_id_format":"^.+$","actions":[]}"#.to_owned(),
            // missing action_id_format
            r#"{"version":"0.4","actions":[]}"#.to_owned(),
        ] {
            let o = Olive::new(false);
            let r = run(
                &NoHometown,
                &o,
                &catalog,
                "0.4",
                "p",
                "q",
                "servicing",
                RUN,
                "session-1",
            );
            assert!(matches!(r, Err(TaskError::Catalog)), "{catalog}");
        }
    }

    #[test]
    fn olive_failure_carries_the_answer() {
        let one = r#"{"action_type":"crm.case_note@1","party_ref":"p","parameters":{"note":"n"},"justification":"j"}"#;
        let o = Olive::new(true);
        match go(&hometown(Some(one)), &o) {
            Err(TaskError::Olive {
                answer,
                context_complete,
                trace_id,
                error,
                ..
            }) => {
                assert_eq!(answer, "display only");
                assert!(context_complete);
                assert_eq!(trace_id, "trace-1");
                assert_eq!(error, OliveError::Unavailable);
            }
            _ => panic!("expected olive failure"),
        }
    }
}
