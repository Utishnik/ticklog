$ErrorActionPreference = 'Stop'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\results_quill'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

# Run 17: ticklog rtrb + Block backpressure vs C++ quill (v12.1.0 baseline and
# v13.0.0). Same parameters as Run 16 (8 MiB rings, RDTSC, BATCH=1000).
$NSPT = '0.313132820'
$RING = '8388608'    # 8 MiB
$THREADS = '1,2,4,8,16'
$SAMPLES = '1000'

$candidates = @(
    @{ exe = 'ticklog_rtrb_block.exe'; name = 'ticklog-rtrb-block'; out = 'windows_ticklog_rtrb_block_r17.json'; extra = @('--chunk-size','64') }
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

# C++ quill harnesses, QUILL_BENCH_QUEUE_CAPACITY=8388608 (8 MiB) each:
# v13.0.0 (bin/quill_harness_8m_v13) and v12.1.0 baseline (bin/quill_harness_8m).
$cppQuills = @(
    @{ exe = 'quill_harness_8m_v13.exe'; out = 'windows_quill_cpp_v13_r17.json' },
    @{ exe = 'quill_harness_8m.exe';     out = 'windows_quill_cpp_r17.json' }
)
foreach ($q in $cppQuills) {
    $cppQuill = Join-Path $binDir $q.exe
    if (Test-Path -LiteralPath $cppQuill) {
        $json = Join-Path $outDir $q.out
        $log  = Join-Path $outDir ($q.out + '.log')
        Remove-Item -LiteralPath $json,$log,"$log.err" -Force -ErrorAction SilentlyContinue
        Write-Output ("=== run " + $q.exe + " 8MiB " + (Get-Date -Format 'HH:mm:ss'))
        $args = @('--ns-per-tick', $NSPT, '--output', $json, '--samples', $SAMPLES)
        $p = Start-Process -FilePath $cppQuill -ArgumentList $args -WorkingDirectory $outDir `
            -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
            -PassThru -WindowStyle Hidden -UseNewEnvironment
        Wait-Process -Id $p.Id -Timeout 900
        if (-not (Test-Path -LiteralPath $json)) { throw "no output for $($q.exe)" }
        Write-Output ("  -> " + $q.out)
    } else {
        Write-Output ("WARN: missing " + $cppQuill + ", skip")
    }
}

Write-Output ('ALL_RUNS_DONE ' + (Get-Date -Format 'HH:mm:ss'))
