$ErrorActionPreference = 'Stop'
$harnessDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog'
$exe = Join-Path $harnessDir 'target\release\ticklog-cross-lang-harness.exe'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
if (-not (Test-Path -LiteralPath $binDir)) { New-Item -ItemType Directory -Path $binDir | Out-Null }

# Run 17: rtrb backend under the build-time Block backpressure policy.
# No hotpath/hotpath feature in this build (cargo tree verified): the
# #[hotpath::measure]/#[hotpath::main] macros expand to no-ops, so the bench
# runs with zero instrumentation overhead.
$builds = @(
    @{ name = 'ticklog_rtrb_block'; features = 'backend-rtrb,policy-block' }
)

foreach ($b in $builds) {
    Write-Output ("=== build " + $b.name + " features=[" + $b.features + "] " + (Get-Date -Format 'HH:mm:ss'))
    Push-Location $harnessDir
    try {
        cargo build --release --features $b.features
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $($b.name)" }
        $dest = Join-Path $binDir ($b.name + '.exe')
        Copy-Item -LiteralPath $exe -Destination $dest -Force
        Write-Output ("  -> " + $dest)
    } finally {
        Pop-Location
    }
}
Write-Output ('ALL_BUILDS_DONE ' + (Get-Date -Format 'HH:mm:ss'))
