# Phase 2 / NONO-01 (D-03, D-12, D-13): ELEVATED, USER-RUN stop/start helper for the dev-registered nono services
# (nono-wfp-service, nono-agentd_*), which run straight from the nono build tree. The user rebuilds and signs in place
# (non-elevated) between -Stop and -Start. Liam never builds, signs or edits in the nono repo. Claude never runs this script.
#   powershell -File scripts/nono-reinstall.ps1 -Stop
#   (rebuild + sign in the nono repo, normal prompt)
#   powershell -File scripts/nono-reinstall.ps1 -Start
# No retry. BFE is never touched. On any unexpected state difference it prints restore commands and exits 1.
# While stopped, no nono session can be started (reduced enforcement window).
param([switch]$Stop, [switch]$Start)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

if (($Stop -and $Start) -or (-not $Stop -and -not $Start)) {
    Write-Output 'REFUSED: pass exactly one of -Stop or -Start.'
    exit 1
}
$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Output 'REFUSED: this script must be run by you from an elevated prompt; it does not self-elevate.'
    exit 1
}

$evidence = Join-Path (Get-Location) 'evidence-local'
New-Item -ItemType Directory -Force -Path $evidence | Out-Null
$beforeFile = Join-Path $evidence 'reinstall-before.json'

function Get-ImagePath($name) {
    $cim = Get-CimInstance -ClassName Win32_Service -Filter ("Name='{0}'" -f $name)
    if (-not $cim) { return $null }
    $pn = $cim.PathName
    if ($pn -match '^\s*"([^"]+)"') { return $Matches[1] }
    return ($pn -split '\s+')[0]
}
function Get-State {
    $svc = @()
    foreach ($pattern in @('nono-wfp-service', 'nono-agentd_*')) {
        foreach ($s in @(Get-Service -Name $pattern -ErrorAction SilentlyContinue)) {
            $img = Get-ImagePath $s.Name
            $sha = if ($img -and (Test-Path -LiteralPath $img)) { (Get-FileHash -LiteralPath $img -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
            $svc += [ordered]@{
                name = $s.Name; status = "$($s.Status)"; start_type = "$($s.StartType)"
                image_file = $(if ($img) { Split-Path -Leaf $img } else { $null })
                image_path_sha256 = $(if ($img) { ([BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes($img))) -replace '-', '').ToLowerInvariant() } else { $null })
                image_sha256 = $sha
            }
        }
    }
    return [ordered]@{ services = $svc }
}
function Write-Restore($items) {
    foreach ($b in $items) {
        $start = switch ($b.start_type) { 'Automatic' { 'auto' } 'Manual' { 'demand' } 'Disabled' { 'disabled' } default { 'demand' } }
        Write-Output ('  sc.exe config {0} start= {1}' -f $b.name, $start)
        if ($b.status -eq 'Running') { Write-Output ('  sc.exe start {0}' -f $b.name) }
    }
}

$exit = 0
if ($Stop) {
    $before = Get-State
    if ($before.services.Count -eq 0) { Write-Output 'REFUSED: no nono services found.'; exit 1 }
    $before | ConvertTo-Json -Depth 5 | Set-Content -Encoding ascii $beforeFile
    try {
        foreach ($s in @(Get-Service -Name 'nono-agentd_*' -ErrorAction SilentlyContinue)) { Stop-Service -Name $s.Name -ErrorAction Stop }
        Stop-Service -Name 'nono-wfp-service' -ErrorAction Stop
        Write-Output 'STOP: services stopped'
    } catch {
        Write-Output ('STOP FAILED: ' + $_.Exception.Message)
        $exit = 1
    } finally {
        $after = Get-State
        $bad = @()
        foreach ($b in $before.services) {
            $a = $after.services | Where-Object { $_.name -eq $b.name }
            if (-not $a -or $a.status -ne 'Stopped' -or $a.start_type -ne $b.start_type) { $bad += $b }
        }
        if ($bad.Count -gt 0) {
            Write-Output 'STATE DIFFERS from expected (Stopped, same start type). No retry. Restore with:'
            Write-Restore $before.services
            $exit = 1
        }
    }
    if ($exit -eq 0) { Write-Output 'PASS stop: both services Stopped, start types unchanged; rebuild and sign, then run with -Start' }
    exit $exit
}

# -Start
if (-not (Test-Path -LiteralPath $beforeFile)) { Write-Output 'REFUSED: reinstall-before.json missing; run -Stop first.'; exit 1 }
$before = Get-Content -Raw -LiteralPath $beforeFile | ConvertFrom-Json
try {
    Start-Service -Name 'nono-wfp-service' -ErrorAction Stop
    foreach ($b in $before.services) {
        if ($b.name -like 'nono-agentd_*') { Start-Service -Name $b.name -ErrorAction Stop }
    }
    Write-Output 'START: services started'
} catch {
    Write-Output ('START FAILED: ' + $_.Exception.Message)
    $exit = 1
} finally {
    $after = Get-State
    $after | ConvertTo-Json -Depth 5 | Set-Content -Encoding ascii (Join-Path $evidence 'reinstall-after.json')
    $bad = @()
    foreach ($b in $before.services) {
        $a = $after.services | Where-Object { $_.name -eq $b.name }
        if (-not $a -or $a.status -ne 'Running' -or $a.start_type -ne $b.start_type -or $a.image_path_sha256 -ne $b.image_path_sha256) { $bad += $b }
    }
    if ($bad.Count -gt 0) {
        Write-Output 'STATE DIFFERS from before the rebuild. No retry. Restore with:'
        Write-Restore $before.services
        $exit = 1
    }
}
if ($exit -eq 0) { Write-Output 'PASS reinstall'; Write-Output 'next: run scripts/nono-diff-probe.ps1' }
exit $exit
