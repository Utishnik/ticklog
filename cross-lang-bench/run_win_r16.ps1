$ErrorActionPreference = 'Stop'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\results_quill'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

$NSPT = '0.313132820'
$RING = '8388608'    # 8 MiB
$THREADS = '1,2,4,8,16'
$SAMPLES = '1000'

$candidates = @(
    @{ exe = 'ticklog_custom_drop.exe'; name = 'ticklog-custom-drop'; out = 'windows_ticklog_custom_drop_r16.json'; extra = @() },
    @{ exe = 'ticklog_rtrb_drop.exe';   name = 'ticklog-rtrb-drop';   out = 'windows_ticklog_rtrb_drop_r16.json';   extra = @('--chunk-size','64') },
    @{ exe = 'ticklog_quill.exe';       name = 'ticklog-quill';       out = 'windows_ticklog_quill_r16.json';       extra = @() },
    @{ exe = 'ticklog_nanolog.exe';     name = 'ticklog-nanolog';     out = 'windows_ticklog_nanolog_r16.json';     extra = @() }
)

foreach ($c in $candidates) {
    $exe = Join-Path $binDir $c.exe
    if (-not (Test-Path -LiteralPath $exe)) { throw "missing $exe" }
    $json = Join-Path $outDir $c.out
    $log  = Join-Path $outDir ($c.out + '.log')
    Remove-Item -LiteralPath $json,$log,"$log.err" -Force -ErrorAction SilentlyContinue
    Write-Output ("=== run " + $c.name + " " + (Get-Date -Format 'HH:mm:ss'))
    $args = @(
        '--ns-per-tick', $NSPT,
        '--output', $json,
        '--candidate', $c.name,
        '--threads', $THREADS,
        '--samples', $SAMPLES,
        '--ring-capacity', $RING
    ) + $c.extra
    $p = Start-Process -FilePath $exe -ArgumentList $args -WorkingDirectory $outDir `
        -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
        -PassThru -WindowStyle Hidden -UseNewEnvironment
    Wait-Process -Id $p.Id -Timeout 900
    if (-not (Test-Path -LiteralPath $json)) { throw "no output for $($c.name)" }
    Write-Output ("  -> " + $c.out)
}

# C++ quill harness rebuilt with QUILL_BENCH_QUEUE_CAPACITY=8388608 (8 MiB)
$cppQuill = Join-Path $binDir 'quill_harness_8m.exe'
if (Test-Path -LiteralPath $cppQuill) {
    $json = Join-Path $outDir 'windows_quill_cpp_r16.json'
    $log  = Join-Path $outDir 'windows_quill_cpp_r16.log'
    Remove-Item -LiteralPath $json,$log,"$log.err" -Force -ErrorAction SilentlyContinue
    Write-Output ('=== run quill-cpp 8MiB ' + (Get-Date -Format 'HH:mm:ss'))
    $args = @('--ns-per-tick', $NSPT, '--output', $json, '--samples', $SAMPLES)
    $p = Start-Process -FilePath $cppQuill -ArgumentList $args -WorkingDirectory $outDir `
        -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
        -PassThru -WindowStyle Hidden -UseNewEnvironment
    Wait-Process -Id $p.Id -Timeout 900
    if (-not (Test-Path -LiteralPath $json)) { throw "no output for quill-cpp" }
    Write-Output '  -> windows_quill_cpp_r16.json'
}

Write-Output ('ALL_RUNS_DONE ' + (Get-Date -Format 'HH:mm:ss'))