$harness = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\rust\ticklog'
$bin = 'C:\Users\Admin\Desktop\ticklog\cross-lang-bench\bin'
$exe = Join-Path $harness 'target\release\ticklog-cross-lang-harness.exe'
$tmp = 'C:\Users\Admin\AppData\Local\Temp\opencode\prof_win_build.log'

function Build-One {
    param([string]$name, [string]$features)
    Write-Output ("=== build " + $name + " [" + $features + "]")
    Push-Location $harness
    & cargo build --release --features $features 2>&1 | Out-File -FilePath $tmp -Encoding utf8
    Pop-Location
    if (Test-Path -LiteralPath $exe) {
        Copy-Item -LiteralPath $exe -Destination (Join-Path $bin ($name + '.exe')) -Force
        Write-Output "  -> $bin\$name.exe"
    } else {
        Write-Output "  BUILD_FAILED for $name"
        Get-Content $tmp -Tail 5
        exit 1
    }
}

Build-One 'ticklog_custom_drop_prof' 'policy-drop,hotpath-profiler'
Build-One 'ticklog_rtrb_drop_prof' 'backend-rtrb,policy-drop,hotpath-profiler'
Build-One 'ticklog_quill_prof' 'policy-quill,hotpath-profiler'
Build-One 'ticklog_nanolog_prof' 'policy-nanolog,hotpath-profiler'
Write-Output 'ALL_PROF_BUILDS_DONE'