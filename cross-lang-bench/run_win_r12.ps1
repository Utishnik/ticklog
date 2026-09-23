$ErrorActionPreference = 'Stop'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\results_quill'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

$NSPT = '0.313132820'
$RING = '67108864'   # 64 MiB
$THREADS = '1,2,4,8,16'
$SAMPLES = '1000'

$ticklog = @(
    @{ exe = 'ticklog_custom.exe';     name = 'ticklog-custom';     out = 'windows_ticklog_custom_r12.json';     extra = @() },
    @{ exe = 'ticklog_rtrb.exe';       name = 'ticklog-rtrb';       out = 'windows_ticklog_rtrb_r12.json';       extra = @('--chunk-size','512') },
    @{ exe = 'ticklog_ringbuf.exe';    name = 'ticklog-ringbuf';    out = 'windows_ticklog_ringbuf_r12.json';    extra = @() },
    @{ exe = 'ticklog_ringbuffer.exe'; name = 'ticklog-ringbuffer'; out = 'windows_ticklog_ringbuffer_r12.json'; extra = @() },
    @{ exe = 'ticklog_triple.exe';     name = 'ticklog-triple';     out = 'windows_ticklog_triple_r12.json';     extra = @() },
    @{ exe = 'ticklog_quill.exe';      name = 'ticklog-quill';      out = 'windows_ticklog_quill_r12.json';      extra = @() },
    @{ exe = 'ticklog_nanolog.exe';    name = 'ticklog-nanolog';    out = 'windows_ticklog_nanolog_r12.json';    extra = @() }
)

foreach ($c in $ticklog) {
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

# C++ quill: only --ns-per-tick / --output / --samples; threads and workloads are fixed
$cpp = Join-Path $binDir '..\cpp\quill\build\quill_harness.exe'
$cpp = [System.IO.Path]::GetFullPath($cpp)
if (-not (Test-Path -LiteralPath $cpp)) { throw "missing $cpp" }
$json = Join-Path $outDir 'windows_quill_cpp_r12.json'
$log  = Join-Path $outDir 'windows_quill_cpp_r12.json.log'
Remove-Item -LiteralPath $json,$log,"$log.err" -Force -ErrorAction SilentlyContinue
Write-Output ("=== run quill-cpp " + (Get-Date -Format 'HH:mm:ss'))
$p = Start-Process -FilePath $cpp -ArgumentList @(
    '--ns-per-tick', $NSPT,
    '--output', $json,
    '--samples', $SAMPLES
) -WorkingDirectory $outDir `
  -RedirectStandardOutput $log -RedirectStandardError "$log.err" `
  -PassThru -WindowStyle Hidden -UseNewEnvironment
Wait-Process -Id $p.Id -Timeout 900
if (-not (Test-Path -LiteralPath $json)) { throw "no output for quill-cpp" }
Write-Output "  -> windows_quill_cpp_r12.json"

Write-Output ('ALL_RUNS_DONE ' + (Get-Date -Format 'HH:mm:ss'))
