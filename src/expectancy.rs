//! Liam's reviewed, static expectations for an AppContainer DirectCli launch.

use crate::receipt::LayerStatus;

pub fn accepts(layer: &str, status: LayerStatus) -> bool {
    use LayerStatus::{Confirmed, EstablishedNotIndependentlyObservable, NotApplicable};
    match layer {
        "AppContainerProfile"
        | "WfpEgressFilters"
        | "JobObjectContainment"
        | "AgentKeyInjection" => status == Confirmed,
        "DaclPackageSidGrant"
        | "DaclAncestorTraverse"
        | "DaclAncestorReadAttrs"
        | "InterpreterCoverageGate" => {
            matches!(status, Confirmed | EstablishedNotIndependentlyObservable)
        }
        "MinifilterAbsence" => status == NotApplicable,
        "RestrictedToken"
        | "MandatoryIntegrityLabel"
        | "DaclSessionSidGrant"
        | "FirewallRulesEgress"
        | "BrokerAuthenticodeTrustGate" => true,
        _ => false,
    }
}

pub const ALL_LAYER_IDS: [&str; 14] = [
    "RestrictedToken",
    "MandatoryIntegrityLabel",
    "AppContainerProfile",
    "DaclSessionSidGrant",
    "DaclPackageSidGrant",
    "DaclAncestorTraverse",
    "DaclAncestorReadAttrs",
    "WfpEgressFilters",
    "FirewallRulesEgress",
    "MinifilterAbsence",
    "JobObjectContainment",
    "BrokerAuthenticodeTrustGate",
    "InterpreterCoverageGate",
    "AgentKeyInjection",
];
