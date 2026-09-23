$ErrorActionPreference = 'Stop'
$harnessDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog'
$exe = Join-Path $harnessDir 'target\release\ticklog-cross-lang-harness.exe'
$binDir = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
if (-not (Test-Path -LiteralPath $binDir)) { New-Item -ItemType Directory -Path $binDir | Out-Null }

$builds = @(
    @{ name = 'ticklog_custom';     features = @() },
    @{ name = 'ticklog_rtrb';       features = @('backend-rtrb') },
    @{ name = 'ticklog_ringbuf';    features = @('backend-ringbuf') },
    @{ name = 'ticklog_ringbuffer'; features = @('backend-ringbuffer') },
    @{ name = 'ticklog_triple';     features = @('backend-triple-buffer') },
    @{ name = 'ticklog_quill';      features = @('policy-quill') },
    @{ name = 'ticklog_nanolog';    features = @('policy-nanolog') }
)

foreach ($b in $builds) {
    Write-Output ("=== build " + $b.name + " features=[" + ($b.features -join ',') + "] " + (Get-Date -Format 'HH:mm:ss'))
    Push-Location $harnessDir
    try {
        if ($b.features.Count -eq 0) {
            cargo build --release
        } else {
            cargo build --release --features ($b.features -join ',')
        }
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $($b.name)" }
        $dest = Join-Path $binDir ($b.name + '.exe')
        Copy-Item -LiteralPath $exe -Destination $dest -Force
        Write-Output ("  -> " + $dest)
    } finally {
        Pop-Location
    }
}
Write-Output ('ALL_BUILDS_DONE ' + (Get-Date -Format 'HH:mm:ss'))
