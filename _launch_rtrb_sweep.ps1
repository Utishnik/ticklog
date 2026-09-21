$ErrorActionPreference='Stop'
$hrx='C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog\target\release\ticklog-cross-lang-harness.exe'
if(-not(Test-Path -LiteralPath $hrx)){ Write-Error 'exe not found' }
$out='C:\Users\Admin\Desktop\ticklog\cross-lang-bench\harness-results\windows\sweep'
if(-not(Test-Path -LiteralPath $out)){ New-Item -ItemType Directory -Path $out -Force | Out-Null }
$json=Join-Path $out 'ticklog_rtrb.json'
$runlog=Join-Path $out 'ticklog_rtrb.run.log'
$errlog="$runlog.err"
$log2=Join-Path $out 'ticklog_rtrb.run.2.log'
$pidf=Join-Path $out 'ticklog_rtrb.harness.pid'
Remove-Item -LiteralPath $json,$runlog,$errlog,$log2,$pidf -Force -ErrorAction SilentlyContinue

# Canonical Windows sweep invocation for the rtrb backend, mirroring the
# existing sweep files (Run-3 methodology):
#   harness --ns-per-tick 0.313097 --output sweep/ticklog_rtrb.json \
#     --candidate ticklog_rtrb --threads 1,2,4,8,16
$arguments=@(
    '--ns-per-tick','0.313097',
    '--output',$json,
    '--candidate','ticklog_rtrb',
    '--threads','1,2,4,8,16'
)

$p=Start-Process -FilePath $hrx -ArgumentList $arguments `
   -WorkingDirectory $out `
   -RedirectStandardOutput $runlog -RedirectStandardError $errlog `
   -PassThru -WindowStyle Hidden
Set-Content -LiteralPath $pidf -Value $p.Id -NoNewline
Write-Output ('pid=' + $p.Id)
Write-Output ('json=' + $json)
Write-Output ('started=' + (Get-Date -Format 'HH:mm:ss'))