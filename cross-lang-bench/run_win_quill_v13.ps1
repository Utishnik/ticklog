$ErrorActionPreference = 'Stop'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\results_quill'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

$NSPT = '0.313132820'
$SAMPLES = '1000'

$exe = Join-Path $binDir 'quill_harness_8m_v13.exe'
if (-not (Test-Path -LiteralPath $exe)) { throw "missing $exe" }
$json = Join-Path $outDir 'windows_quill_cpp_v13_r16.json'
$log  = Join-Path $outDir 'windows_quill_cpp_v13_r16.log'
Remove-Item -LiteralPath $json,$log,"$log.err" -Force -ErrorAction SilentlyContinue
Write-Output ('=== run quill-cpp v13 8MiB ' + (Get-Date -Format 'HH:mm:ss'))
$args = @('--ns-per-tick', $NSPT, '--output', $json, '--samples', $SAMPLES)
$p = Start-Process -FilePath $exe -ArgumentList $args -WorkingDirectory $outDir `
    -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
    -PassThru -WindowStyle Hidden -UseNewEnvironment
Wait-Process -Id $p.Id -Timeout 900
if (-not (Test-Path -LiteralPath $json)) { throw "no output for quill-cpp v13" }
Write-Output '  -> windows_quill_cpp_v13_r16.json'
Write-Output ('WIN_V13_DONE ' + (Get-Date -Format 'HH:mm:ss'))