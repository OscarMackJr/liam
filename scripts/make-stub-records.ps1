param(
    [Parameter(Mandatory = $true)] [string] $OutputDirectory,
    [Parameter(Mandatory = $true)] [int] $LiamPid,
    [Parameter(Mandatory = $true)] [int] $ProxyPort,
    [string] $TokenArm = "WriteRestricted",
    [long] $NowUnix = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null

$layers = @(
    @{ id = "RestrictedToken"; status = "Unconfirmed" },
    @{ id = "MandatoryIntegrityLabel"; status = "NotApplicable" },
    @{ id = "AppContainerProfile"; status = "Confirmed" },
    @{ id = "DaclSessionSidGrant"; status = "NotApplicable" },
    @{ id = "DaclPackageSidGrant"; status = "Confirmed" },
    @{ id = "DaclAncestorTraverse"; status = "Confirmed" },
    @{ id = "DaclAncestorReadAttrs"; status = "Confirmed" },
    @{ id = "WfpEgressFilters"; status = "Confirmed" },
    @{ id = "FirewallRulesEgress"; status = "NotApplicable" },
    @{ id = "MinifilterAbsence"; status = "NotApplicable" },
    @{ id = "JobObjectContainment"; status = "Confirmed" },
    @{ id = "BrokerAuthenticodeTrustGate"; status = "NotApplicable" },
    @{ id = "InterpreterCoverageGate"; status = "Confirmed" },
    @{ id = "AgentKeyInjection"; status = "Confirmed" }
)

$receipt = [ordered]@{
    schema_version = 2
    session_id = "stub-session-1"
    pid = $LiamPid
    entry_path = "DirectCli"
    token_arm = $TokenArm
    outcome = "Ran"
    layers = $layers
}
$agent = [ordered]@{
    schema_version = 1
    session_id = "stub-session-1"
    agent_id = "liam"
    purpose = "servicing"
    key_id = "0192a3b4-c5d6-7e8f-9a0b-1c2d3e4f5a6b"
    key_expires_at_unix = $NowUnix + 3600
    injected_on = @("hometown", "olive")
    direct_gateway = $false
    profile_version = 4
    proxy_port = $ProxyPort
    written_at_unix = $NowUnix
}

$receipt | ConvertTo-Json -Depth 4 | Set-Content -NoNewline (Join-Path $OutputDirectory "receipt.json")
$agent | ConvertTo-Json -Depth 4 | Set-Content -NoNewline (Join-Path $OutputDirectory "agent.json")
(Get-Item -LiteralPath (Join-Path $OutputDirectory "receipt.json")).IsReadOnly = $true
(Get-Item -LiteralPath (Join-Path $OutputDirectory "agent.json")).IsReadOnly = $true
