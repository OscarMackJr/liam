# D-08: verify this session runs with a standard-user (non-elevated) token. No secrets, no identity output.
# Run from the repository root: powershell -NoProfile -File scripts/check-standard-user.ps1
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
$isAdminRole = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$groups = (& (Join-Path $env:SystemRoot 'System32\whoami.exe') /groups) -join "`n"
$isHigh = $groups -match 'High Mandatory Level'

if ($isAdminRole -or $isHigh) {
    Write-Output 'FAIL D-08: elevated or administrator token; stop and ask the user'
    exit 1
}
Write-Output 'PASS D-08: standard-user token (not elevated)'
exit 0
