pub mod clients;
pub mod config;
pub mod evidence;
pub mod expectancy;
pub mod receipt;
pub mod task;

#[cfg(feature = "stubs")]
pub mod stubs;

#[cfg(test)]
mod tests {
    use super::receipt::{validate_startup, StartupPolicy};

    #[test]
    fn accepts_a_valid_supervisor_record_pair() {
        let receipt = r#"{
          "schema_version":2,"session_id":"session-1","pid":42,
          "entry_path":"DirectCli","token_arm":"WriteRestricted","outcome":"Ran",
          "layers":[
            {"id":"RestrictedToken","status":"Unconfirmed"},{"id":"MandatoryIntegrityLabel","status":"NotApplicable"},
            {"id":"AppContainerProfile","status":"Confirmed"},{"id":"DaclSessionSidGrant","status":"NotApplicable"},
            {"id":"DaclPackageSidGrant","status":"Confirmed"},{"id":"DaclAncestorTraverse","status":"Confirmed"},
            {"id":"DaclAncestorReadAttrs","status":"Confirmed"},{"id":"WfpEgressFilters","status":"Confirmed"},
            {"id":"FirewallRulesEgress","status":"NotApplicable"},{"id":"MinifilterAbsence","status":"NotApplicable"},
            {"id":"JobObjectContainment","status":"Confirmed"},{"id":"BrokerAuthenticodeTrustGate","status":"NotApplicable"},
            {"id":"InterpreterCoverageGate","status":"Confirmed"},{"id":"AgentKeyInjection","status":"Confirmed"}
          ]
        }"#;
        let agent = r#"{
          "schema_version":1,"session_id":"session-1","agent_id":"liam",
          "purpose":"servicing","key_id":"0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
          "key_expires_at_unix":1000,"injected_on":["hometown","olive"],
          "direct_gateway":false,"profile_version":4,"proxy_port":50413,"written_at_unix":100
        }"#;
        let policy =
            StartupPolicy::for_stub("liam", "servicing", "WriteRestricted", 4, 120, 90, 30);
        let result = validate_startup(receipt, agent, &policy, 42, 100, 50413);
        assert!(
            result.is_ok(),
            "valid supervisor-owned record pair must pass: {result:?}"
        );
    }

    #[test]
    fn rejects_an_unconfirmed_agent_key_layer_before_any_network_work() {
        let receipt = valid_receipt().replace(
            "{\"id\":\"AgentKeyInjection\",\"status\":\"Confirmed\"}",
            "{\"id\":\"AgentKeyInjection\",\"status\":\"Unconfirmed\"}",
        );
        let result = validate_startup(&receipt, &valid_agent(), &policy(), 42, 100, 50413);
        assert_eq!(result.unwrap_err().0, "receipt:layer:AgentKeyInjection");
    }

    #[test]
    fn rejects_an_agent_record_for_a_different_session() {
        let agent = valid_agent().replace("\"session-1\"", "\"other-session\"");
        let result = validate_startup(&valid_receipt(), &agent, &policy(), 42, 100, 50413);
        assert_eq!(result.unwrap_err().0, "agent_record:session_id");
    }

    #[test]
    fn rejects_the_remaining_critical_record_mutations() {
        let cases = [
            (
                valid_receipt().replace("\"schema_version\":2", "\"schema_version\":1"),
                valid_agent(),
                "receipt:schema_version",
            ),
            (
                valid_receipt().replace("\"outcome\":\"Ran\"", "\"outcome\":\"Refused\""),
                valid_agent(),
                "receipt:outcome",
            ),
            (
                valid_receipt()
                    .replace("\"entry_path\":\"DirectCli\"", "\"entry_path\":\"Broker\""),
                valid_agent(),
                "receipt:entry_path",
            ),
            (
                valid_receipt().replace("\"pid\":42", "\"pid\":43"),
                valid_agent(),
                "receipt:pid",
            ),
            (
                valid_receipt().replace(
                    "\"token_arm\":\"WriteRestricted\"",
                    "\"token_arm\":\"LowIlPrimary\"",
                ),
                valid_agent(),
                "receipt:token_arm",
            ),
            (
                valid_receipt(),
                valid_agent().replace("\"purpose\":\"servicing\"", "\"purpose\":\"analytics\""),
                "agent_record:purpose",
            ),
            (
                valid_receipt(),
                valid_agent().replace("\"profile_version\":4", "\"profile_version\":3"),
                "agent_record:profile_version",
            ),
            (
                valid_receipt(),
                valid_agent().replace("\"direct_gateway\":false", "\"direct_gateway\":true"),
                "agent_record:direct_gateway",
            ),
        ];
        for (receipt, agent, expected) in cases {
            assert_eq!(
                validate_startup(&receipt, &agent, &policy(), 42, 100, 50413)
                    .unwrap_err()
                    .0,
                expected
            );
        }
    }

    fn policy() -> StartupPolicy {
        StartupPolicy::for_stub("liam", "servicing", "WriteRestricted", 4, 120, 90, 30)
    }

    fn valid_agent() -> String {
        r#"{
          "schema_version":1,"session_id":"session-1","agent_id":"liam",
          "purpose":"servicing","key_id":"0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b",
          "key_expires_at_unix":1000,"injected_on":["hometown","olive"],
          "direct_gateway":false,"profile_version":4,"proxy_port":50413,"written_at_unix":100
        }"#
        .to_owned()
    }

    fn valid_receipt() -> String {
        r#"{
          "schema_version":2,"session_id":"session-1","pid":42,
          "entry_path":"DirectCli","token_arm":"WriteRestricted","outcome":"Ran",
          "layers":[
            {"id":"RestrictedToken","status":"Unconfirmed"},{"id":"MandatoryIntegrityLabel","status":"NotApplicable"},
            {"id":"AppContainerProfile","status":"Confirmed"},{"id":"DaclSessionSidGrant","status":"NotApplicable"},
            {"id":"DaclPackageSidGrant","status":"Confirmed"},{"id":"DaclAncestorTraverse","status":"Confirmed"},
            {"id":"DaclAncestorReadAttrs","status":"Confirmed"},{"id":"WfpEgressFilters","status":"Confirmed"},
            {"id":"FirewallRulesEgress","status":"NotApplicable"},{"id":"MinifilterAbsence","status":"NotApplicable"},
            {"id":"JobObjectContainment","status":"Confirmed"},{"id":"BrokerAuthenticodeTrustGate","status":"NotApplicable"},
            {"id":"InterpreterCoverageGate","status":"Confirmed"},{"id":"AgentKeyInjection","status":"Confirmed"}
          ]
        }"#.to_owned()
    }
}
