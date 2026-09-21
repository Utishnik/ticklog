$ErrorActionPreference = 'Stop'
$hrx = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog\target\release\ticklog-cross-lang-harness.exe'
$out = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\harness-results\windows\sweep'
if (-not (Test-Path -LiteralPath $out)) { New-Item -ItemType Directory -Path $out -Force | Out-Null }
$json = Join-Path $out 'ticklog_rtrb.json'
$log  = Join-Path $out 'ticklog_rtrb.harness.log'
$err  = Join-Path $out 'ticklog_rtrb.harness.log.stderr'
$pidf = Join-Path $out 'ticklog_rtrb.harness.pid'
Remove-Item -LiteralPath $json,$log,$err,$pidf -Force -ErrorAction SilentlyContinue

$ns = '0.313097'
$argsList = @(
    '--ns-per-tick', $ns,
    '--output', $json,
    '--candidate', 'ticklog_rtrb',
    '--threads', '1,2,4,8,16'
)

# Detach: Start-Process (new process, not a job), redirect to files,
# write the PID so a separate poller can watch for exit.
$proc = Start-Process -FilePath $hrx `
    -ArgumentList $argsList `
    -WorkingDirectory $out `
    -RedirectStandardOutput $log `
    -RedirectStandardError $err `
    -PassThru -WindowStyle Hidden -UseNewEnvironment

Set-Content -LiteralPath $pidf -Value $proc.Id -NoNewline
Write-Output ('pid=' + $proc.Id)
Write-Output ('json=' + $json)
Write-Output ('started=' + (Get-Date -Format 'HH:mm:ss'))
Write-Output ('args=' + ($argsList -join ' '))
