$ErrorActionPreference = 'Stop'
$hrx = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog\target\release\ticklog-cross-lang-harness.exe'
if (-not (Test-Path -LiteralPath $hrx)) { Write-Error "harness exe not found: $hrx" }

$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\harness-results\windows\sweep'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

$json  = Join-Path $outDir 'ticklog_rtrb.json'
$log   = Join-Path $outDir 'ticklog_rtrb.run.log'
$pidf  = Join-Path $outDir 'ticklog_rtrb.run.pid'
$errlog = "$log.err"

Remove-Item -LiteralPath $json,$log,"$log.err" -Force -ErrorAction SilentlyContinue

# Canonical sweep for the rtrb (rtrb-crate SPSC) backend, mirroring the
# existing ticklog_ringbuf.json sweep exactly (same ns_per_tick calibration,
# same threads 1/2/4/8/16, batch 1000 -> 10M total messages, 10k samples).
$la = @(
    '--ns-per-tick', '0.313097',
    '--output',      $json,
    '--candidate',   'ticklog_rtrb',
    '--threads',     '1,2,4,8,16'
)

$pr = Start-Process -FilePath $hrx -ArgumentList $la -WorkingDirectory $outDir `
     -RedirectStandardOutput $log -RedirectStandardError $errlog `
     -PassThru -WindowStyle Hidden -UseNewEnvironment

Set-Content -LiteralPath $pidf -Value $pr.Id -NoNewline
Write-Output ('pid=' + $pr.Id)
Write-Output ('json=' + $json)
Write-Output ('log=' + $log)
Write-Output ('started=' + (Get-Date -Format 'HH:mm:ss'))
