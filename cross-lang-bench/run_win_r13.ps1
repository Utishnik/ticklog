$ErrorActionPreference = 'Stop'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$outDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\results_quill'
if (-not (Test-Path -LiteralPath $outDir)) { New-Item -ItemType Directory -Path $outDir | Out-Null }

$NSPT = '0.313132820'
$RING = '67108864'   # 64 MiB (user asked 60 MB; capacity must be power-of-two)
$THREADS = '1,2,4,8,16'
$SAMPLES = '1000'

$candidates = @(
    @{ exe = 'ticklog_custom_drop.exe'; name = 'ticklog-custom-drop'; out = 'windows_ticklog_custom_drop_r13.json'; extra = @() },
    @{ exe = 'ticklog_rtrb_drop.exe';   name = 'ticklog-rtrb-drop';   out = 'windows_ticklog_rtrb_drop_r13.json';   extra = @('--chunk-size','64') },
    @{ exe = 'ticklog_quill.exe';       name = 'ticklog-quill';       out = 'windows_ticklog_quill_r13.json';       extra = @() },
    @{ exe = 'ticklog_nanolog.exe';     name = 'ticklog-nanolog';     out = 'windows_ticklog_nanolog_r13.json';     extra = @() }
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
Write-Output ('ALL_RUNS_DONE ' + (Get-Date -Format 'HH:mm:ss'))
