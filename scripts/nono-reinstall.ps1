# Phase 2 / NONO-01 (D-03, D-12, D-13): ELEVATED, USER-RUN reinstall of the nono CLI, WFP service and driver
# from a machine-scope MSI built and signed from the pinned nono commit using the procedure documented in the
# nono repo (README "Install"; scripts/build-windows-msi.ps1; scripts/sign-poc-local.ps1).
# Liam never builds, signs or edits in the nono repo. Claude never runs this script.
#   powershell -File scripts/nono-reinstall.ps1 -MsiPath <machine MSI built from the pin>
# No retry. BFE is never touched. On any unexpected state difference it prints restore commands and exits 1.
param([Parameter(Mandatory = $true)][string]$MsiPath)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Output 'REFUSED: this script must be run by you from an elevated prompt; it does not self-elevate.'
    exit 1
}
if (-not (Test-Path -LiteralPath $MsiPath) -or $MsiPath -notmatch '\.msi$') {
    Write-Output 'REFUSED: -MsiPath must point to an existing machine-scope .msi built from the pinned commit.'
    exit 1
}

$installDir = Join-Path $env:ProgramFiles 'nono'
$evidence = Join-Path (Get-Location) 'evidence-local'
New-Item -ItemType Directory -Force -Path $evidence | Out-Null

function Get-State {
    $svc = @()
    foreach ($pattern in @('nono-wfp-service', 'nono-agentd_*')) {
        foreach ($s in @(Get-Service -Name $pattern -ErrorAction SilentlyContinue)) {
            $svc += [ordered]@{ name = $s.Name; status = "$($s.Status)"; start_type = "$($s.StartType)" }
        }
    }
    $hashes = [ordered]@{}
    foreach ($f in @('nono.exe', 'nono-wfp-service.exe', 'nono-wfp-driver.sys')) {
        $p = Join-Path $installDir $f
        $hashes[$f] = if (Test-Path -LiteralPath $p) { (Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash.ToLowerInvariant() } else { $null }
    }
    return [ordered]@{ services = $svc; sha256 = $hashes }
}

$before = Get-State
$before | ConvertTo-Json -Depth 5 | Set-Content -Encoding ascii (Join-Path $evidence 'reinstall-before.json')
$exit = 0
try {
    $log = Join-Path $evidence 'reinstall-msi.log'
    $proc = Start-Process -FilePath "$env:SystemRoot\System32\msiexec.exe" -Wait -PassThru `
        -ArgumentList @('/i', "`"$MsiPath`"", '/qn', '/l*v', "`"$log`"")
    if ($proc.ExitCode -ne 0 -and $proc.ExitCode -ne 3010) { throw "msiexec exit code $($proc.ExitCode)" }
    Write-Output 'INSTALL: msiexec completed'
} catch {
    Write-Output ('INSTALL FAILED: ' + $_.Exception.Message)
    $exit = 1
} finally {
    $after = Get-State
    $after | ConvertTo-Json -Depth 5 | Set-Content -Encoding ascii (Join-Path $evidence 'reinstall-after.json')
    $bad = @()
    foreach ($b in $before.services) {
        $a = $after.services | Where-Object { $_.name -eq $b.name }
        if (-not $a) { $bad += $b }
        elseif ($a.status -ne $b.status -or $a.start_type -ne $b.start_type) { $bad += $b }
    }
    if ($bad.Count -gt 0) {
        Write-Output 'STATE DIFFERS from before the reinstall. No retry. Restore with:'
        foreach ($b in $bad) {
            $start = switch ($b.start_type) { 'Automatic' { 'auto' } 'Manual' { 'demand' } 'Disabled' { 'disabled' } default { 'demand' } }
            Write-Output ('  sc.exe config {0} start= {1}' -f $b.name, $start)
            if ($b.status -eq 'Running') { Write-Output ('  sc.exe start {0}' -f $b.name) }
        }
        $exit = 1
    }
}
if ($exit -eq 0) { Write-Output 'PASS reinstall: service state matches prior state; run scripts/nono-diff-probe.ps1 next' }
exit $exit
