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

---

# Windows: собственный CLI-профилировщик (Run 18.2)

На Windows доступного профилировщика не было: WPR/WPA требуют админа
(попытка `0xc5585011`), `VSPerfCmd` в VS2022 отсутствует, VerySleepy — GUI и
не фильтрует по CPU. Написан `winprof/profiler_cli.c` (~500 строк, dbghelp) и
прогнаны те же 4 сценария, что и в WSL-части.

## Методика

- Сэмплер: `SuspendThread` всех потоков цели → `GetThreadContext` +
  `StackWalk64` → `SymFromAddr`. Символы: PDB харнесса рядом с exe +
  `srv*...msdl`; модули preload'ятся вручную (`EnumProcessModules` +
  `SymLoadModuleEx`), `SYMOPT_DEFERRED_LOADS` выключен (иначе PDB exe не
  находится, `err=126`).
- On-CPU-фильтр: дельты `GetThreadTimes` с накоплением — сэмпл только если
  поток реально жёг CPU с прошлого раунда; в отчёт идёт точный `cpu_ms`/`cpu%`
  по каждому потоку (не сэмплы).
- Реальный интервал ~8–11 мс вместо заданных 5: `Sleep(5)` без
  `timeBeginPeriod(1)` спит 15.6 мс; `CreateToolhelp32Snapshot` стоил 42 мс
  и 83% времени раунда (список потоков обновляется раз в 20 раундов);
  резолв символов кешируется по PC (раньше ~44 мс/раунд).
- Прогон: `winprof/run_win_cli_prof.ps1`, 4 сценария (render/raw × t1/t8),
  `--ns-per-tick 0.313132820 --ring-capacity 8388608 --chunk-size 64`,
  render `--samples 20000`, raw `400000`. Каждый харнесс гонит 3 workload'а
  подряд (single_int, mixed, string) — отчёт агрегирует все три; границы фаз
  видны по группам producer-потоков.
- Артефакты: `winprof/*.prof.txt` (header, top self%, threads с cpu_ms и
  top-5 функций, топ-40 полных стеков), `winprof/*.json` (throughput/p50/p99),
  категории self% — `winprof/sum_cats.ps1`.

| отчёт | окно | rounds | samples | всего CPU процесса |
|---|---|---|---|---|
| render8m_t1 | 39.8 с | 4601 | 3924  | 82.5 с (2.1 ядра)  |
| render8m_t8 | 60 с (лимит) | 5003 | 26649 | 532 с (8.9 ядер)   |
| raw8m_t1    | 32.7 с | 3901 | 3259  | 69.5 с (2.1 ядра)  |
| raw8m_t8    | 26.8 с | 2296 | 11356 | 253 с (9.4 ядра)   |

## Результат 1. Drain — однопоточный потолок (везде)

cpu_ms из секции threads:

| сценарий | drain cpu / окно | утилизация drain | доля drain от всего CPU |
|---|---|---|---|
| render t1 | 41.5 с / 39.8 с | **104%**, почти без yield | 50.3% |
| render t8 | 60.8 с / 60 с  | **101%**                     | 11.4% |
| raw t1    | 34.6 с / 32.7 с | 106%, но 52.6% сэмплов drain — idle-yield | 49.8% |
| raw t8    | 28.4 с / 26.8 с | **106%**, memmove+decode, без простоя | 11.3% |

- **render**: drain не простаивает — кольцо всегда полное, producer'ы ждут
  (`SwitchToThread`), чистый consumer-bound.
- **raw t1**: наоборот producer-bound — drain 52.6% сэмплов в
  `NtDelayExecution` (idle-yield `drain.rs:344`, кольцо пустое), один
  producer упирается в ~47 M r/s.
- **raw t8**: 8 producer'ов заполняют кольцо, drain гонит memmove — потолок
  ~47 M r/s: single_int 47.1 → 47.4 M (+0.7%), а mixed/string, где
  producer-код тяжелее, выигрывают от параллелизма: +22.5% и +33%.

## Результат 2. Куда идёт CPU (категории self%)

| категория | render t1 | render t8 | raw t1 | raw t8 |
|---|---|---|---|---|
| yield/backoff (`SwitchToThread`) | 47.0% | **86.7%** | 28.2% | 65.2% |
| heap (`RtlAllocateHeap/Free`, `GetProcessHeap`) | 19.0% | 1.6% | — | — |
| форматирование (`format.rs`, `core::fmt`, flt2dec) | 9.5% | 0.5% | 1.8% | 0.5% |
| `drain_ring` (декод/разбор) | 6.7% | 0.2% | 12.2% | 5.0% |
| string-рост (`raw_vec`, `from_utf8_lossy`) | 1.3% | — | 1.5% | 0.4% |
| memmove/memset | 0.6% | 0.1% | 4.0% | 3.1% |
| producer-dispatch (харнесс/макрос) | 1.5% | 0.5% | 41.5% | 16.2% |
| `VCRUNTIME140!_NLG*` (миссимволизация) | 2.4% | 0.2% | 5.5% | 3.3% |

Суммы неточны на ~10–11%: печатаются строки self% ≥0.05%, диффузный хвост
render-пути в таблицу не попадает.

- render t1, топ-5 функций самого drain: `KERNEL32!GetProcessHeap` 1.5%,
  `RtlAllocateHeap+0x19ac` 1.3%, `RtlAllocateHeap+0x186d` 1.3%,
  `RtlFreeHeap+0x183` 1.3%, `RtlFreeHeap+0xf8` 1.2% — heap-цепочка на каждую
  аллокацию (Rust на Windows живёт в process heap) в топ-5; форматирование
  размазано по десяткам мелких функций, каждая <0.5%.
- raw t8, топ-5 drain: `VCRUNTIME140!memmove` 14.9%, `drain_ring+0x76d`
  10.8%, `drain_ring+0x789` 7.9%, vcruntime-хелперы 7.8+4.0% — memmove из
  `staging.drain(..pos)` (drain.rs:967) + декод.

## Результат 3. Backoff на Windows жжёт ядра

Топ-1 self всего процесса: `ntdll!NtDelayExecution+0x14` — в render t8
**83.2%** (это `SwitchToThread` из `std::thread::yield_now`, `src/backoff.rs`
после 64 pause). Полные стеки: три фазовых producer-closure (+0x1366/+0x1dd6/
+0x896) = 44.9% всех сэмплов render t1 и 82.9% render t8.

- render t8: **86.7% всего CPU процесса — producer-backoff в ядре**, полезная
  работа (drain) — 11.4%; 461 с из 532 с CPU уходят в yield-ожидание.
- Масштабирование render **отрицательное**: t8 медленнее t1 на 30–38%
  (single_int 1.67→1.16, mixed 1.02→0.67, string 1.80→1.11 M r/s), p50 вырос
  в 1.4–1.7× — 8 producer'ов добавляют только contention кольца и
  yield-трафика, drain по-прежнему один поток.
- На WSL этого не было видно: producer-closure 5.10%, ожидание уходило туда,
  куда user-space perf не смотрит. Сэмплер, берущий стеки и в ядре, показал
  всю картину.

## Сравнение с WSL (выводы Run 18.1 подтверждены)

- Consumer-bound подтверждён на обеих ОС: WSL — drain ~70% incl, Windows —
  drain = 100% одного ядра при простаивающих producer'ах.
- Состав drain-работы совпадает: heap + форматирование. WSL `malloc+cfree`
  10.6%, Windows heap-цепочка — 19% всех сэмплов и весь топ-5 drain;
  `GetProcessHeap` на каждую аллокацию дороже glibc malloc.
- Форматирование: WSL ~32% чётко по функциям, Windows ~9.5% + большой
  диффузный хвост — та же проблема, другие имена.
- raw-потолок ~47 M r/s на Windows (memmove+decode) сопоставим с WSL
  3.5–18.7 M r/s — упирается не в producer.

## Ограничения

- Отчёт агрегирует 3 workload'а; разбить по фазам можно только по группам tid.
- render t8 не уложился в 60-с окно профилировщика (json полный, отчёт —
  первые 60 с).
- Сэмпл ≈ 1 на поток-раунд с CPU-дельтой; точные доли — по cpu_ms, сэмплы
  дают распределение внутри потока (у busy-потоков топ ограничен
  256 distinct-функциями).

## Что дальше (по Windows-данным)

1. Подтверждается приоритет Run18: аллокации (heap 19%!) и форматирование в
   drain — убрать `format!`-фолбэки/`from_utf8_lossy`, inline-буфер вместо
   String.
2. Backoff: yield в ядро дорог на Windows — при полном кольце можно
   перейти на event/wait-примитив или ограничить число ждущих; но это лишь
   снижает потери, потолок в drain.
3. render не масштабируется на 8 producer'ов (−30…−38%): контеншн кольца —
   либо принимать t1-режим как оптимум, либо sharded-кольца.
