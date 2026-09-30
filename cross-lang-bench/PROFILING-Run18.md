# Профилирование ticklog через cross-lang-bench (WSL, Run 18.1)

Все замеры — `cross-lang-bench/rust/ticklog` (features `backend-rtrb,policy-block`),
perf 7.0.14 в WSL, `/home/utishnik/tickperf/*.data`. Машина: AMD Ryzen 7 7735HS,
WSL2 (hypervisor, TSC-оверлей), `ns_per_tick = 0.313148759`.

## Инструментарий

- `perf_report.sh`, `perf_report2.sh`, `perf_annotate.sh`, `perf_src.sh`,
  `perf_hotlines.sh`, `perf_minpct.sh`, `perf_sections.sh` — обёртки над perf/awk
  (awk/python в `wsl -e bash -c` ломаются PowerShell, поэтому всё через sh-файлы).
- `ab_compare.py` — сравнение двух bench-JSON (p50/throughput дельты).
- `perf/rdtsc_probe.c`, `perf/rdtsc_probe2.c` — пробники стоимости rdtsc.
- Профили: `8m.data` (drain-bound, render), `raw8m.data` (`--sink-raw`),
  `dbg.data` (producer-bound), `quill_v13.data` (контроль: quill v13).

## Узкое место 1. Drain-limited: рендер + аллокации (главное)

Сustained throughput через drain ограничен рендером, а не кольцом:

| конфигурация | t1 throughput, рандом workload |
|---|---|
| render (Null без RawLogSink) | 1.5–2.0 M r/s |
| `--sink-raw` (raw-обход) | 3.5–18.7 M r/s |
| producer-only, 512MiB кольцо | 36.9 M r/s |

perf `8m.data` (drain-bound, 8MiB кольцо, render-режим), ~70% drain-потока:

- `ticklog::drain::drain_ring` 26.27%
- `__memmove_avx_unaligned_erms` 11.16%  ← в т.ч. `staging.drain(..pos)` (drain.rs:967)
- `malloc` 5.60% + `cfree` 4.97%         ← `format!`-фолбэки и `from_utf8_lossy`
- форматирование ~32% суммарно (`format_with_spec` 3.58, `format_unsigned_impl`
  3.48, grisu `format_shortest_opt` 3.47, `core::fmt::write` 3.41, `format_inner`
  3.27, `String::write_str` 2.42, `format_str` 2.29, `pad_integral` 1.92 и т.д.)
- producer-closure 5.10% (в producer-bound `dbg.data` — 40.07%, см. узкое место 3)

perf `raw8m.data` (`--sink-raw`): producer-closure 31.59%, `drain_ring` 8.19%,
memmove 4.06%, `raw_sink` 3.05% → рендер и аллокации и были стоимостью drain.

**A/B throughput (стабильно между двумя прогонами):** string t1 +1124%
(1.53→18.7M), mixed t1 +755% (1.21→10.4M), single_int t1 +107–116% (1.80→3.54M).
p50-диффы в пределах шума (run-to-run p50 до ±50%).

**Харнесс-баг:** `BenchSink::Null` (main.rs:137) не реализует `RawLogSink`,
поэтому drain рендерит каждую строку впустую. Добавлен флаг `--sink-raw`
(BenchSink::Raw → `ticklog::NullSink`, у которого `raw_sink = Some(self)`).

**Кандидаты оптимизации в ticklog:**
- `src/format.rs:745–889` — `format!`-фолбэки аллоцируют String на каждый аргумент;
- `src/drain.rs:235–241` — `fmt_str` → `String::from_utf8_lossy` (аллокация на lossy);
- `src/drain.rs:967` — `staging.drain(..pos)` (memmove-источник, 11% cycles);
- сырой путь для null/битых строк без рендера.

## Узкое место 2. RDTSC в producer-пути (~64% p50)

Эксперимент: `raw_timestamp()` временно заменён константой (timestamp.rs откатан,
`git diff` чист, бинарь пересобран):

| workload | t1 p50 (render) | t1 p50 (no-rdtsc) | Δp50 |
|---|---|---|---|
| single_int | 26.12 | 9.43 | **−63.9%** |
| mixed | 45.21 | 30.89 | −31.7% |
| string | 27.35 | 30.36 | +11% (шум) |

Пробники: bare `rdtsc` ≈ **49–50 cycles/iter** (dependent и 4-way unrolled),
`lfence;rdtsc` = 93.48 cycles. На этой виртуализованной машине чтение TSC стоит
~15 ns, т.е. почти всё p50 single_int. Атрибуция perf (`dbg.data`, producer-bound,
63.5% producer-closure) — три сайта `movq %fs:-0x80, %r13` сразу после `rdtsc`
(23.00/21.90/18.65%), сам `rdtsc` 0.01% → классический skid/slow-retire после
долгой инструкции, а не TLS.

**Вывод:** на этом железе timestamp — доминанта producer-latency; gain от
оптимизации самого dispatch ограничен (без rdtsc осталось 9.4 ns).
Кандидаты: batch-чтение timestamp, `rdtscp`-оптимизация, либо смириться —
контр-quill тоже платит за timestamp (quill v13 p50 ниже в 29/30 клеток).

## Узкое место 3. Форматирование в drain (общее с quill)

`quill_v13.data` (45K samples): `QuillBackend` 72.42% / harness 27.58%;
горячее: `_exit` 14.91%, `SpinBarrier` 15.54% (спешение main-потоков),
`_process_lowest_timestamp_transit_event` 15.13%, formatting ~19–21%
(fmt write 11.86, PatternFormatter 3.89, fill 3.13) — consumer-bound, как ticklog.

Quill тоже аллоцирует/рендерит строки в backend — значит A/B `--sink-raw`
показывает нечестный (слишком лестный) сценарий, а render-режим — честный
относительно quill. Для сравнения двух библиотек держим render как baseline,
raw — как верхний потолок.

## Ограничения атрибуции

- debuginfo в финальный бинарь не попал (`readelf -S | grep debug` пуст,
  `addr2line` → `?`, в annot.txt 0 строк `.rs`) — построчная атрибуция
  возможна только после пересборки с `-Cdebuginfo=1` + проверка секций.
- Профили `results_quill/*_prof.out` — Windows-прогоны `run_quill_profiles.ps1`
  (не стейджить).

## Что дальше (кандидаты)

1. Убрать аллокации из рендера (`format.rs`-фолбэки, `from_utf8_lossy`) —
   самый большой честный выигрыш drain-пути.
2. Пересборка с debuginfo → построчная карта `drain_ring`/`format`.
3. Решение по timestamp: batch/`rdtscp`/принять как есть.
