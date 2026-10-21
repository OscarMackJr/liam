# TC-BUILD-01 / TC-BUILD-02 release-build checks. No network, no secrets.
# Run from the repository root: powershell -NoProfile -File scripts/measure-build.ps1
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

cargo build --locked --offline --release
if ($LASTEXITCODE -ne 0) { Write-Output 'FAIL build: cargo build --release failed'; exit 1 }

$exe = Join-Path (Get-Location) 'target\release\liam.exe'
if (-not (Test-Path $exe)) { Write-Output 'FAIL build: target\release\liam.exe missing'; exit 1 }

# TC-BUILD-01: stub strings must be absent from the release binary.
$bytes = [System.IO.File]::ReadAllBytes($exe)
$latin1 = [System.Text.Encoding]::GetEncoding(28591).GetString($bytes)
$forbidden = @('LIAM_STUB_GATEWAY_ATTEMPT', 'gateway.invalid', 'LIAM_STUBS', 'LIAM_STUB_PID', 'tiny_http')
$present = @()
foreach ($needle in $forbidden) {
    if ($latin1.Contains($needle)) { $present += $needle }
}
if ($present.Count -gt 0) {
    foreach ($needle in $present) { Write-Output "FAIL TC-BUILD-01: release binary contains '$needle'" }
    exit 1
}
Write-Output ('PASS TC-BUILD-01: none of {0} present in release liam.exe' -f ($forbidden -join ', '))

# TC-BUILD-02: imported DLLs (evidence only).
$tool = Get-Command dumpbin -ErrorAction SilentlyContinue
if (-not $tool) {
    $roots = @("${env:ProgramFiles}\Microsoft Visual Studio", "${env:ProgramFiles(x86)}\Microsoft Visual Studio") |
        Where-Object { $_ -and (Test-Path $_) }
    foreach ($root in $roots) {
        $found = Get-ChildItem -Path $root -Recurse -Filter dumpbin.exe -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match 'Hostx64\\x64' } | Select-Object -First 1
        if ($found) { $tool = $found.FullName; break }
    }
} else {
    $tool = $tool.Source
}
if (-not $tool) {
    Write-Output 'NOTE TC-BUILD-02: dumpbin unavailable; import list not recorded'
    exit 0
}
$imports = & $tool /imports $exe
$dlls = $imports | Where-Object { $_ -match '^\s+\S+\.dll\s*$' } | ForEach-Object { $_.Trim() } | Sort-Object -Unique
Write-Output 'TC-BUILD-02 imported DLLs:'
$dlls | ForEach-Object { Write-Output "  $_" }
$crt = $dlls | Where-Object { $_ -match '^(vcruntime|msvcp|ucrtbase|api-ms-win-crt)' }
if ($crt) {
    Write-Output ('FAIL TC-BUILD-02: CRT DLL imported: {0}' -f ($crt -join ', '))
    exit 1
}
Write-Output 'PASS TC-BUILD-02: no VCRUNTIME/UCRT import (static CRT); system DLLs listed above'
exit 0
