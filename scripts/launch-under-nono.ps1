[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Party,
    [Parameter(Mandatory)][string]$Question,
    [Parameter(Mandatory)][string]$Purpose,
    [string]$Profile = "profiles/liam.nono.toml",
    [string]$Config = "liam.toml"
)

$ErrorActionPreference = 'Stop'

# This script deliberately delegates confinement, record paths and proxy
# environment construction to nono. It never accepts or supplies a credential.
if (-not (Get-Command nono -ErrorAction SilentlyContinue)) {
    throw 'nono executable was not found on PATH.'
}
if (-not (Test-Path -LiteralPath $Profile)) { throw "Profile not found: $Profile" }
if (-not (Test-Path -LiteralPath $Config)) { throw "Configuration not found: $Config" }

nono run --profile $Profile --purpose $Purpose -- `
    .\target\release\liam.exe --purpose $Purpose --party $Party --question $Question
