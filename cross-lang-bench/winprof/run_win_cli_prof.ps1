# Запуск 4 профилей ticklog_rtrb_block под profiler_cli.exe.
# Собрать: build_all.cmd (VS2022 x64). Отчёты пишутся рядом со скриптом.
$ErrorActionPreference = 'Continue'
$out = $PSScriptRoot
$bin = Join-Path (Split-Path $PSScriptRoot -Parent) 'bin'
$exe = "$bin\ticklog_rtrb_block.exe"
$sympath = "$bin;srv*$out\symcache*https://msdl.microsoft.com/download/symbols"
$prof = "$out\profiler_cli.exe"

$scenarios = @(
    @{ name='render8m_t8'; samples='20000';  extra=@('--threads','8') },
    @{ name='render8m_t1'; samples='20000';  extra=@('--threads','1') },
    @{ name='raw8m_t8';    samples='400000'; extra=@('--threads','8','--sink-raw') },
    @{ name='raw8m_t1';    samples='400000'; extra=@('--threads','1','--sink-raw') }
)

foreach ($sc in $scenarios) {
    $name = $sc.name
    Write-Host "=== $name ==="
    $args = @('--ns-per-tick','0.313132820','--ring-capacity','8388608',
              '--chunk-size','64','--samples',$sc.samples) +
            $sc.extra + @('--output', "$out\$name.json")
    $p = Start-Process $exe -ArgumentList $args -PassThru -WorkingDirectory $bin -WindowStyle Hidden -RedirectStandardError "$out\$name.err"
    Start-Sleep -Seconds 3
    if ($p.HasExited) {
        Write-Host "  harness exited early"
        Get-Content "$out\$name.err" -ErrorAction SilentlyContinue | Select-Object -First 5
        continue
    }
    $null = & $prof "$($p.Id)" 60 5 "$out\$name.prof.txt" $sympath 2>&1
    Write-Host "  profiler exit=$LASTEXITCODE"
    if (-not $p.HasExited) { $null = $p.WaitForExit(300000) }
    Write-Host "  harness done"
    Start-Sleep -Milliseconds 300
}
Write-Host "ALL_DONE"
