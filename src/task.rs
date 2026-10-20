use crate::clients::{
    hometown::{AnswerRequest, HometownClient, HometownError},
    olive::{OliveClient, OliveError, ProposalRequest},
};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct Catalog {
    version: String,
    actions: Vec<CatalogAction>,
}
#[derive(Debug, Deserialize)]
struct CatalogAction {
    action_type: String,
    required_parameters: Vec<String>,
}
#[derive(Debug)]
pub enum TaskError {
    Hometown(HometownError),
    Olive(OliveError),
}
pub struct TaskResult {
    pub answer: String,
    pub context_complete: bool,
    pub trace_id: String,
    pub proposal: Option<(String, String)>,
    pub suppressed: Option<&'static str>,
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
    let catalog: Catalog = serde_json::from_str(catalog_json)
        .map_err(|_| TaskError::Olive(OliveError::InvalidResponse))?;
    if catalog.version != catalog_version {
        return Err(TaskError::Olive(OliveError::InvalidResponse));
    }
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
    let Some(proposal) = answer.proposal.clone() else {
        return Ok(TaskResult {
            answer: answer.answer,
            context_complete: answer.context_complete,
            trace_id: answer.trace_id,
            proposal: None,
            suppressed: None,
        });
    };
    if proposal.party_ref != party_ref {
        return Ok(TaskResult {
            answer: answer.answer,
            context_complete: answer.context_complete,
            trace_id: answer.trace_id,
            proposal: None,
            suppressed: Some("schema"),
        });
    }
    let Some(definition) = catalog
        .actions
        .iter()
        .find(|entry| entry.action_type == proposal.action_type)
    else {
        return Ok(TaskResult {
            answer: answer.answer,
            context_complete: answer.context_complete,
            trace_id: answer.trace_id,
            proposal: None,
            suppressed: Some("not_in_catalog"),
        });
    };
    let object = match proposal.parameters.as_object() {
        Some(object) => object,
        None => {
            return Ok(TaskResult {
                answer: answer.answer,
                context_complete: answer.context_complete,
                trace_id: answer.trace_id,
                proposal: None,
                suppressed: Some("schema"),
            })
        }
    };
    let required: HashSet<_> = definition
        .required_parameters
        .iter()
        .map(String::as_str)
        .collect();
    if !required.iter().all(|key| object.contains_key(*key))
        || object.keys().any(|key| !required.contains(key.as_str()))
    {
        return Ok(TaskResult {
            answer: answer.answer,
            context_complete: answer.context_complete,
            trace_id: answer.trace_id,
            proposal: None,
            suppressed: Some("schema"),
        });
    }
    let response = olive
        .submit(&ProposalRequest {
            proposal: &proposal,
            run_id,
            trace_ref: &answer.trace_id,
        })
        .map_err(TaskError::Olive)?;
    Ok(TaskResult {
        answer: answer.answer,
        context_complete: answer.context_complete,
        trace_id: answer.trace_id,
        proposal: Some((response.proposal_id, response.status)),
        suppressed: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::hometown::{AnswerResponse, Proposal};
    use crate::clients::olive::ProposalResponse;
    use serde_json::json;

    struct Hometown {
        response: AnswerResponse,
    }
    impl HometownClient for Hometown {
        fn answer(&self, _: &AnswerRequest<'_>) -> Result<AnswerResponse, HometownError> {
            Ok(self.response.clone())
        }
    }
    struct Olive;
    impl OliveClient for Olive {
        fn submit(&self, request: &ProposalRequest<'_>) -> Result<ProposalResponse, OliveError> {
            assert_eq!(request.trace_ref, "trace-1");
            Ok(ProposalResponse {
                proposal_id: "proposal-1".into(),
                status: "awaiting_approval".into(),
                reason: None,
            })
        }
    }

    #[test]
    fn submits_only_a_catalog_valid_proposal() {
        let hometown = Hometown {
            response: AnswerResponse {
                trace_id: "trace-1".into(),
                answer: "display only".into(),
                context_complete: true,
                proposal: Some(Proposal {
                    action_type: "crm.case_note@1".into(),
                    party_ref: "p".into(),
                    parameters: json!({"note":"n"}),
                    justification: "display only".into(),
                }),
            },
        };
        let result = run(&hometown, &Olive, r#"{"version":"0.4","actions":[{"action_type":"crm.case_note@1","required_parameters":["note"]}]}"#, "0.4", "p", "q", "servicing", "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b", "session-1").unwrap();
        assert_eq!(result.proposal.unwrap().1, "awaiting_approval");
    }

    #[test]
    fn suppresses_a_proposal_outside_the_catalog() {
        let hometown = Hometown {
            response: AnswerResponse {
                trace_id: "trace-1".into(),
                answer: "display only".into(),
                context_complete: true,
                proposal: Some(Proposal {
                    action_type: "unknown@1".into(),
                    party_ref: "p".into(),
                    parameters: json!({}),
                    justification: "display only".into(),
                }),
            },
        };
        let result = run(
            &hometown,
            &Olive,
            r#"{"version":"0.4","actions":[]}"#,
            "0.4",
            "p",
            "q",
            "servicing",
            "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
            "session-1",
        )
        .unwrap();
        assert_eq!(result.suppressed, Some("not_in_catalog"));
    }
}
