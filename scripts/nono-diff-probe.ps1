# Phase 2 / NONO-01: read-only probe of the pinned nono build vs the RUNNING dev-registered CLI, services and driver.
# Non-elevated. Read-only git only. Content-free output: versions, hashes, service names/status. No paths, SIDs or users.
# Services run straight from the nono build tree (ImagePath), so a hash match to the build is trivial; evidence is
# freshness (LastWriteTime after pin commit), signature, restart-after-build, and --version.
#   powershell -NoProfile -File scripts/nono-diff-probe.ps1 -PinnedCommit <40-hex> [-PreInstall]
# -PreInstall downgrades expected pre-rebuild failures (stale build, not restarted, unsigned) to NOTE.
param(
    [Parameter(Mandatory = $true)][string]$PinnedCommit,
    [switch]$PreInstall
)
$ErrorActionPreference = 'Stop'
$nonoRepo = Join-Path $env:USERPROFILE 'Nono'
$script:failed = $false

function Out-Line($level, $text) {
    Write-Output ('{0} {1}' -f $level, $text)
    if ($level -eq 'FAIL') { $script:failed = $true }
}
function Check($ok, $text, $failText) {
    if ($ok) { Out-Line 'PASS' $text }
    elseif ($PreInstall) { Out-Line 'NOTE' "$failText (expected before rebuild)" }
    else { Out-Line 'FAIL' $failText }
}
function Get-Sha($path) {
    if (-not $path -or -not (Test-Path -LiteralPath $path)) { return $null }
    return (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}
function Get-ImagePath($pathName) {
    if ($pathName -match '^\s*"([^"]+)"') { return $Matches[1] }
    return ($pathName -split '\s+')[0]
}

# Pin and clean tree.
$head = (git -C $nonoRepo rev-parse HEAD 2>$null)
if ($PinnedCommit -notmatch '^[0-9a-f]{40}$') { Out-Line 'FAIL' 'pin: -PinnedCommit is not a full 40-character hash' }
elseif ($head -eq $PinnedCommit) { Out-Line 'PASS' "nono commit: $head" }
else { Out-Line 'FAIL' 'nono commit: repository HEAD does not equal the pinned commit' }
$dirty = (git -C $nonoRepo status --porcelain --untracked-files=no 2>$null)
if ([string]::IsNullOrEmpty($dirty)) { Out-Line 'PASS' 'nono tree: tracked files clean' }
else { Out-Line 'FAIL' 'nono tree: tracked files have uncommitted changes' }
$pinTime = $null
if ($PinnedCommit -match '^[0-9a-f]{40}$') {
    $iso = (git -C $nonoRepo show -s --format=%cI $PinnedCommit 2>$null)
    if ($iso) { $pinTime = [DateTimeOffset]::Parse($iso).UtcDateTime; Out-Line 'NOTE' ('pin committer time (UTC): {0:s}Z' -f $pinTime) }
    else { Out-Line 'FAIL' 'pin: commit time not readable' }
}

# Pinned Cargo version.
$pinVersion = $null
$cargo = Join-Path $nonoRepo 'crates\nono-cli\Cargo.toml'
if (Test-Path -LiteralPath $cargo) {
    $m = Select-String -LiteralPath $cargo -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if ($m) { $pinVersion = $m.Matches[0].Groups[1].Value }
}
if ($pinVersion) { Out-Line 'NOTE' "pinned Cargo version: $pinVersion" } else { Out-Line 'FAIL' 'pinned Cargo version not readable' }

# Windows build: ver + UBR.
$cv = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$ver = (& "$env:SystemRoot\System32\cmd.exe" /c ver) | Where-Object { $_ } | Select-Object -First 1
Out-Line 'NOTE' ('ver + UBR: {0}.{1}' -f $ver.Trim(), $cv.UBR)

function Check-Binary($label, $path, $procStart, $signRequired = $true) {
    if (-not $path -or -not (Test-Path -LiteralPath $path)) { Out-Line 'FAIL' "$label binary: missing"; return }
    $item = Get-Item -LiteralPath $path
    Out-Line 'NOTE' "$label sha256: $(Get-Sha $path)"
    if ($pinTime) {
        Check ($item.LastWriteTimeUtc -gt $pinTime) "$label built after pin commit" "$label stale build (older than pin commit)"
    }
    $sig = Get-AuthenticodeSignature -LiteralPath $path
    $thumb = if ($sig.SignerCertificate) { $sig.SignerCertificate.Thumbprint.Substring(0, 8) } else { $null }
    if ($sig.Status -eq 'Valid') { Out-Line 'PASS' "$label signature Valid (signer $thumb)" }
    elseif ($sig.Status -eq 'NotSigned' -and -not $signRequired) { Out-Line 'NOTE' "$label NotSigned (nono's sign-poc-local.ps1 does not sign it)" }
    elseif ($sig.Status -eq 'NotSigned') { Check $false '' "$label NotSigned" }
    elseif ($sig.SignerCertificate -and $sig.SignerCertificate.Subject -eq $sig.SignerCertificate.Issuer) {
        Out-Line 'PASS' "$label signed by self-signed POC cert (signer $thumb, status $($sig.Status))"
    } else { Out-Line 'FAIL' "$label signature status $($sig.Status)" }
    if ($procStart) {
        Check ($procStart -gt $item.LastWriteTimeUtc) "$label process started after binary write" "$label process predates binary (not restarted onto new build)"
    }
}

# CLI on PATH.
$cmd = Get-Command nono -ErrorAction SilentlyContinue
$buildCli = Join-Path $nonoRepo 'target\release\nono.exe'
if ($cmd) {
    if ((Test-Path -LiteralPath $buildCli) -and ((Get-Item $cmd.Source).FullName -eq (Get-Item $buildCli).FullName)) { Out-Line 'NOTE' 'nono.exe on PATH is the target\release build' }
    else { Out-Line 'NOTE' 'nono.exe on PATH is another copy, not target\release' }
    $v = ((& $cmd.Source --version) 2>&1 | Select-Object -First 1) -as [string]
    Out-Line 'NOTE' "nono --version: $v"
    if ($pinVersion) { Check ($v -match [regex]::Escape($pinVersion)) "nono --version matches pinned $pinVersion" "nono --version does not match pinned $pinVersion" }
    Check-Binary 'nono.exe' $cmd.Source $null
} else { Out-Line 'FAIL' 'nono.exe not found on PATH' }

# Services: resolve binary from ImagePath.
$progFiles = Join-Path $env:ProgramFiles 'nono'
foreach ($pattern in @('nono-wfp-service', 'nono-agentd_*')) {
    $svcs = @(Get-Service -Name $pattern -ErrorAction SilentlyContinue)
    if ($svcs.Count -eq 0) { Out-Line 'NOTE' "service ${pattern}: not installed"; continue }
    foreach ($s in $svcs) {
        Out-Line 'NOTE' ('service {0}: {1} / {2}' -f $s.Name, $s.Status, $s.StartType)
        $cim = Get-CimInstance Win32_Service -Filter ("Name='{0}'" -f $s.Name)
        $img = Get-ImagePath $cim.PathName
        $procStart = $null
        if ($cim.ProcessId -gt 0) {
            $p = Get-Process -Id $cim.ProcessId -ErrorAction SilentlyContinue
            if ($p -and $p.StartTime) { $procStart = $p.StartTime.ToUniversalTime() }
            if (-not $procStart) {
                $cp = Get-CimInstance Win32_Process -Filter ("ProcessId={0}" -f $cim.ProcessId) -ErrorAction SilentlyContinue
                if ($cp -and $cp.CreationDate) { $procStart = $cp.CreationDate.ToUniversalTime() }
            }
            if (-not $procStart) { Out-Line 'NOTE' "$($s.Name) process start time unreadable: restart check skipped" }
        } else { Out-Line 'NOTE' "$($s.Name) not running: restart check skipped" }
        Check-Binary ("$($s.Name) [" + (Split-Path -Leaf $img) + ']') $img $procStart ($pattern -ne 'nono-agentd_*')
        if ($img -like "$progFiles*") { Out-Line 'NOTE' "$($s.Name) runs from the Program Files install" }
        else { Out-Line 'NOTE' "$($s.Name) runs from a dev registration outside Program Files (Program Files copy, if any, is unused)" }
    }
}

# Driver: checked-in .sys vs the installed driver found via the system-driver registration.
$built = Join-Path $nonoRepo 'crates\nono-cli\data\windows\nono-wfp-driver.sys'
$drv = @(Get-CimInstance Win32_SystemDriver -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'nono*' })
$drvPath = $null
if ($drv.Count -gt 0) {
    $raw = $drv[0].PathName -replace '^\\\?\?\\', '' -replace '^\\SystemRoot', $env:SystemRoot
    if (Test-Path -LiteralPath $raw) { $drvPath = $raw }
    Out-Line 'NOTE' "driver registration: $($drv[0].Name) / $($drv[0].State)"
} else { Out-Line 'NOTE' 'driver registration: no nono system driver found' }
$instSha = Get-Sha $drvPath
$builtSha = Get-Sha $built
if (-not $instSha) { Out-Line 'NOTE' 'nono-wfp-driver.sys installed: not located via registration' }
elseif (-not $builtSha) { Out-Line 'NOTE' 'nono-wfp-driver.sys MATCH: checked-in artifact absent, not compared' }
else {
    Out-Line 'NOTE' "nono-wfp-driver.sys installed sha256: $instSha"
    Out-Line 'NOTE' "nono-wfp-driver.sys checked-in sha256: $builtSha"
    Check ($instSha -eq $builtSha) 'nono-wfp-driver.sys MATCH' 'nono-wfp-driver.sys MATCH: installed differs from checked-in'
}

if ($script:failed) { exit 1 }
exit 0
