#![cfg(feature = "stubs")]

use liam::clients::hometown::Proposal;
use liam::clients::hometown::{AnswerRequest, HometownClient, HometownError, HttpHometownClient};
use liam::clients::olive::{HttpOliveClient, OliveClient, OliveError, ProposalRequest};
use liam::stubs::{CaptureProxy, HometownStub, OliveStub};
use serde_json::json;

#[test]
fn hometown_request_has_no_authorization_before_proxy_injection() {
    let hometown = HometownStub::answer_without_proposal();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let client = HttpHometownClient::new(hometown.base_url(), proxy.url(), 2).unwrap();

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
}

#[test]
fn olive_request_has_no_authorization_before_proxy_injection() {
    let olive = OliveStub::awaiting_approval();
    let proxy = CaptureProxy::forward_once(olive.base_url(), "/v1/proposals");
    let client = HttpOliveClient::new(olive.base_url(), proxy.url(), 2).unwrap();
    let proposal = Proposal {
        action_type: "crm.case_note@1".into(),
        party_ref: "opaque-party".into(),
        parameters: json!({"note":"x"}),
        justification: "display only".into(),
    };

    let response = client
        .submit(&ProposalRequest {
            proposal: &proposal,
            run_id: "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
            trace_ref: "trace-1",
        })
        .unwrap();

    assert_eq!(response.status, "awaiting_approval");
    assert!(!proxy.captured().has_authorization);
    assert!(olive.captured().body.contains("trace_ref"));
    assert_eq!(
        olive.captured().authorization.as_deref(),
        Some("Bearer proxy-sentinel")
    );
}

#[test]
fn hometown_server_error_is_classified_as_retryable_unavailability() {
    let hometown = HometownStub::server_error();
    let proxy = CaptureProxy::forward_once_to(hometown.base_url());
    let client = HttpHometownClient::new(hometown.base_url(), proxy.url(), 2).unwrap();
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
        (401, "{}", HometownError::NoInjectedKey),
        (
            401,
            r#"{"code":"key_inactive"}"#,
            HometownError::KeyInactive,
        ),
        (403, "{}", HometownError::PurposeRefused),
        (429, "{}", HometownError::Status(429)),
    ] {
        let hometown = HometownStub::status(status, body);
        let proxy = CaptureProxy::forward_once_to(hometown.base_url());
        let client = HttpHometownClient::new(hometown.base_url(), proxy.url(), 2).unwrap();
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
    let proposal = Proposal {
        action_type: "crm.case_note@1".into(),
        party_ref: "opaque-party".into(),
        parameters: json!({"note":"x"}),
        justification: "display only".into(),
    };
    let refused = OliveStub::status(
        200,
        r#"{"proposal_id":"p-1","status":"refused_policy","reason":"policy"}"#,
    );
    let proxy = CaptureProxy::forward_once(refused.base_url(), "/v1/proposals");
    let client = HttpOliveClient::new(refused.base_url(), proxy.url(), 2).unwrap();
    assert_eq!(
        client
            .submit(&ProposalRequest {
                proposal: &proposal,
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
        let client = HttpOliveClient::new(olive.base_url(), proxy.url(), 2).unwrap();
        assert_eq!(
            client
                .submit(&ProposalRequest {
                    proposal: &proposal,
                    run_id: "run",
                    trace_ref: "trace"
                })
                .unwrap_err(),
            OliveError::Status(status)
        );
    }
}
