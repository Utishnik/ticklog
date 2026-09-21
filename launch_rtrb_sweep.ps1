$ErrorActionPreference = 'Stop'
$exe = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog\target\release\ticklog-cross-lang-harness.exe'
if (-not (Test-Path -LiteralPath $exe)) {
    Write-Error "harness exe not found: $exe"
    exit 1
}
$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\harness-results\windows\sweep'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }
$json = Join-Path $outDir 'ticklog_rtrb.json'
$log  = Join-Path $outDir 'ticklog_rtrb.run.log'
Remove-Item -LiteralPath $json,$log -Force -ErrorAction SilentlyContinue

# Canonical Windows sweep args (matches ticklog_ringbuf.json methodology):
# ns_per_tick calibration from the existing sweep JSON header.
$args = @(
    '--ns-per-tick', '0.313097',
    '--output', $json,
    '--candidate', 'ticklog_rtrb',
    '--threads', '1,2,4,8,16',
    '--chunk-size', '512'
)
$p = Start-Process -FilePath $exe -ArgumentList $args -WorkingDirectory $outDir `
    -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
    -PassThru -WindowStyle Hidden -UseNewEnvironment
Set-Content -LiteralPath "$log.pid" -Value $p.Id
Write-Output ('pid=' + $p.Id)
Write-Output ('started=' + (Get-Date -Format 'HH:mm:ss'))
Write-Output ('json=' + $json)
