$bin = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$out = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\results_quill'
$NSPT = '0.313132820'
$RING = '8388608'
$THREADS = '1,2,4,8,16'
$SAMP = '1000'

$cands = @(
    @{ n = 'custom_drop'; e = @() },
    @{ n = 'rtrb_drop';   e = @('--chunk-size', '64') },
    @{ n = 'quill';       e = @() },
    @{ n = 'nanolog';     e = @() }
)

foreach ($c in $cands) {
    Write-Output ('=== ' + $c.n + ' ' + (Get-Date -Format 'HH:mm:ss'))
    $exe = Join-Path $bin ('ticklog_' + $c.n + '_prof.exe')
    $json = Join-Path $out ('windows_ticklog_' + $c.n + '_prof.json')
    $args = @(
        '--ns-per-tick', $NSPT,
        '--output', $json,
        '--candidate', ('ticklog-' + $c.n),
        '--threads', $THREADS,
        '--samples', $SAMP,
        '--ring-capacity', $RING
    ) + $c.e
    $p = Start-Process -FilePath $exe -ArgumentList $args -WorkingDirectory $out `
        -RedirectStandardOutput (Join-Path $out ('windows_ticklog_' + $c.n + '_prof.out')) `
        -RedirectStandardError (Join-Path $out ('windows_ticklog_' + $c.n + '_prof.log')) `
        -PassThru -WindowStyle Hidden -UseNewEnvironment
    Wait-Process -Id $p.Id -Timeout 900
}
Write-Output 'ALL_PROF_RUNS_DONE'