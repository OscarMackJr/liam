# Phase 2 / NONO-01: read-only probe of the pinned nono build vs the installed CLI, service and driver.
# Non-elevated. Read-only git only. Content-free output: versions, hashes, service names/status. No paths, SIDs or users.
# Run from the repository root:
#   powershell -NoProfile -File scripts/nono-diff-probe.ps1 -PinnedCommit <40-hex> [-PreInstall]
# -PreInstall downgrades installed-vs-build mismatches to NOTE (expected before the reinstall).
param(
    [Parameter(Mandatory = $true)][string]$PinnedCommit,
    [switch]$PreInstall
)
$ErrorActionPreference = 'Stop'
$nonoRepo = Join-Path $env:USERPROFILE 'Nono'
$installDir = Join-Path $env:ProgramFiles 'nono'
$script:failed = $false

function Out-Line($level, $text) {
    Write-Output ('{0} {1}' -f $level, $text)
    if ($level -eq 'FAIL') { $script:failed = $true }
}
function Get-Sha($path) {
    if (-not (Test-Path -LiteralPath $path)) { return $null }
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}

# Pin and clean tree (read-only git commands only).
$head = (git -C $nonoRepo rev-parse HEAD 2>$null)
if ($PinnedCommit -notmatch '^[0-9a-f]{40}$') { Out-Line 'FAIL' 'pin: -PinnedCommit is not a full 40-character hash' }
elseif ($head -eq $PinnedCommit) { Out-Line 'PASS' "nono commit: $head" }
else { Out-Line 'FAIL' 'nono commit: repository HEAD does not equal the pinned commit' }
$dirty = (git -C $nonoRepo status --porcelain --untracked-files=no 2>$null)
if ([string]::IsNullOrEmpty($dirty)) { Out-Line 'PASS' 'nono tree: tracked files clean' }
else { Out-Line 'FAIL' 'nono tree: tracked files have uncommitted changes' }

# Windows build: ver + UBR.
$cv = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$ver = (& "$env:SystemRoot\System32\cmd.exe" /c ver) | Where-Object { $_ } | Select-Object -First 1
Out-Line 'NOTE' ('ver + UBR: {0}.{1}' -f $ver.Trim(), $cv.UBR)

# Installed CLI.
$cmd = Get-Command nono -ErrorAction SilentlyContinue
if ($cmd) {
    $v = (& $cmd.Source --version) 2>&1 | Select-Object -First 1
    Out-Line 'NOTE' "nono --version: $v"
} else { Out-Line 'FAIL' 'nono.exe not found on PATH' }

# Installed binaries vs the build from the pin.
$cliPath = if ($cmd) { $cmd.Source } else { Join-Path $installDir 'nono.exe' }
$pairs = @(
    @{ Name = 'nono.exe';             Installed = $cliPath; Built = Join-Path $nonoRepo 'target\release\nono.exe' },
    @{ Name = 'nono-wfp-service.exe'; Installed = Join-Path $installDir 'nono-wfp-service.exe'; Built = Join-Path $nonoRepo 'target\release\nono-wfp-service.exe' },
    @{ Name = 'nono-wfp-driver.sys';  Installed = Join-Path $installDir 'nono-wfp-driver.sys';  Built = Join-Path $nonoRepo 'crates\nono-cli\data\windows\nono-wfp-driver.sys' }
)
foreach ($p in $pairs) {
    $inst = Get-Sha $p.Installed
    $built = Get-Sha $p.Built
    if (-not $inst) { Out-Line 'FAIL' "$($p.Name) installed: missing"; continue }
    Out-Line 'NOTE' "$($p.Name) installed sha256: $inst"
    if (-not $built) { Out-Line 'NOTE' "$($p.Name) MATCH: pinned build artifact absent, not compared"; continue }
    Out-Line 'NOTE' "$($p.Name) built sha256: $built"
    if ($inst -eq $built) { Out-Line 'PASS' "$($p.Name) MATCH" }
    elseif ($PreInstall) { Out-Line 'NOTE' "$($p.Name) MATCH: differs from pinned build (expected before reinstall)" }
    else { Out-Line 'FAIL' "$($p.Name) MATCH: installed differs from pinned build" }
}

# Services: names, status, start type only.
foreach ($pattern in @('nono-wfp-service', 'nono-agentd_*')) {
    $svcs = @(Get-Service -Name $pattern -ErrorAction SilentlyContinue)
    if ($svcs.Count -eq 0) { Out-Line 'NOTE' "service ${pattern}: not installed"; continue }
    foreach ($s in $svcs) { Out-Line 'NOTE' ('service {0}: {1} / {2}' -f $s.Name, $s.Status, $s.StartType) }
}

if ($script:failed) { exit 1 }
exit 0
