# Суммирование self% по категориям из *.prof.txt (путь — каталог скрипта).
Get-ChildItem $PSScriptRoot\*.prof.txt | ForEach-Object {
    $file = $_.Name
    $c = Get-Content $_.FullName
    $s = ($c | Select-String -SimpleMatch '== top self').LineNumber
    $e = ($c | Select-String -SimpleMatch '== threads').LineNumber
    $rows = $c[($s)..($e-2)] | Where-Object { $_ -match '^\s+[\d.]+%' }
    $cats = @{}
    foreach ($r in $rows) {
        if ($r -notmatch '^\s+([\d.]+)%\s+[\d.]+%\s+\d+\s+(.+)$') { continue }
        $pct = [double]$Matches[1]; $fn = $Matches[2]
        $cat =
            if ($fn -match 'DelayExecution|SwitchToThread') { 'yield_backoff' }
            elseif ($fn -match 'RtlAllocateHeap|RtlFreeHeap|GetProcessHeap|HeapFree|HeapAlloc|process_heap_alloc') { 'heap' }
            elseif ($fn -match 'ticklog::format|core::fmt|alloc::fmt|flt2dec|float_to_decimal|pad_integral|write_str|format_inner') { 'format' }
            elseif ($fn -match 'raw_vec|do_reserve|finish_grow|from_utf8|str::lossy|string::impl|String::') { 'string_grow' }
            elseif ($fn -match 'ticklog::drain') { 'drain_ring' }
            elseif ($fn -match 'memmove|memcpy|memset') { 'memmove_memset' }
            elseif ($fn -match 'measure_config|ticklog_cross_lang_harness|macros::dispatch') { 'producer_dispatch' }
            elseif ($fn -match '_NLG_Return2') { 'vcruntime_nlg' }
            else { 'other' }
        $cats[$cat] = [double]$cats[$cat] + $pct
    }
    "## $file"
    $cats.GetEnumerator() | Sort-Object Value -Descending | ForEach-Object { "  {0,-18} {1,7:N2}%" -f $_.Key, $_.Value }
}
