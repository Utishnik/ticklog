// CLI sampling profiler for Windows (no admin required).
// Suspends all target threads at a fixed interval, walks each stack with
// dbghelp StackWalk64, resolves symbols from local PDBs / symbol server,
// aggregates self + inclusive sample counts per function and per thread.
//
// Build (VS2022 x64 Native Tools):
//   cl /O2 /W3 profiler_cli.c /link dbghelp.lib
//
// Usage:
//   profiler_cli <pid> <seconds> <interval_ms> <out.txt> [sym_search_path]

#define _WIN32_WINNT 0x0601
#include <windows.h>
#include <tlhelp32.h>
#include <dbghelp.h>
#include <psapi.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#pragma comment(lib, "dbghelp.lib")
#pragma comment(lib, "psapi.lib")
#pragma comment(lib, "winmm.lib")

#define MAX_STACK   64
#define MAX_NAME    256
#define MAX_THREADS 256
#define MAX_AGG     16384

typedef struct {
    char key[MAX_NAME + 64];
    unsigned long long self;
    unsigned long long incl;
} SymAgg;

static SymAgg g_agg[MAX_AGG];
static int    g_nagg;

static DWORD  g_tids[MAX_THREADS];
static char   g_tnames[MAX_THREADS][64];
static unsigned long long g_thread_samples[MAX_THREADS];
static int    g_nthreads_seen;

/* previous kernel+user CPU time (100ns units) per tracked thread, for on-CPU filter */
static unsigned long long g_prev_cpu[MAX_THREADS];
static unsigned long long g_acc_cpu[MAX_THREADS];  /* accumulated since last counted sample */
static int g_have_prev[MAX_THREADS];

/* per-thread top functions (fixed slots, linear scan) */
#define TTOP 256
#define TTOP_NAME 200
static char g_tfn_name[MAX_THREADS][TTOP][TTOP_NAME];
static unsigned long long g_tfn_n[MAX_THREADS][TTOP];
static int g_tfn_cnt[MAX_THREADS];

static CRITICAL_SECTION g_lock;
static HANDLE g_process;
static DWORD  g_interval_ms;
static unsigned long long g_total_samples;

static int agg_find(const char *key) {
    for (int i = 0; i < g_nagg; i++)
        if (strcmp(g_agg[i].key, key) == 0) return i;
    if (g_nagg >= MAX_AGG) return -1;
    strncpy(g_agg[g_nagg].key, key, sizeof(g_agg[0].key) - 1);
    g_agg[g_nagg].key[sizeof(g_agg[0].key) - 1] = 0;
    g_agg[g_nagg].self = g_agg[g_nagg].incl = 0;
    return g_nagg++;
}

static int __cdecl read_mem(HANDLE hproc, DWORD64 base, LPVOID buf, DWORD size, LPDWORD nread) {
    SIZE_T got = 0;
    if (!ReadProcessMemory(hproc, (LPCVOID)(uintptr_t)base, buf, size, &got)) return 0;
    if (nread) *nread = (DWORD)got;
    return 1;
}

static void thread_name_of(HANDLE hthread, DWORD tid, char *out, size_t n) {
    out[0] = 0;
    typedef HRESULT (WINAPI *GTD)(HANDLE, PWSTR *);
    static GTD gtd = NULL;
    static int resolved = 0;
    if (!resolved) {
        HMODULE k = GetModuleHandleW(L"kernel32.dll");
        gtd = (GTD)GetProcAddress(k, "GetThreadDescription");
        resolved = 1;
    }
    if (gtd) {
        PWSTR wdesc = NULL;
        if (SUCCEEDED(gtd(hthread, &wdesc)) && wdesc && wdesc[0]) {
            WideCharToMultiByte(CP_UTF8, 0, wdesc, -1, out, (int)n - 1, NULL, NULL);
            out[n - 1] = 0;
        }
        if (wdesc) LocalFree(wdesc);
    }
    if (!out[0]) snprintf(out, n, "tid_%lu", tid);
}

static char g_symbuf[sizeof(SYMBOL_INFO) + MAX_NAME];

/* pc -> formatted "mod!sym+disp" cache; PCs repeat heavily across rounds */
#define PC_CACHE 65536
static struct { DWORD64 pc; char name[MAX_NAME]; } g_pcc[PC_CACHE];

/* phase timings, ms accumulated */
static unsigned long long g_t_snap, g_t_susp, g_t_walk, g_t_agg;

static void resolve_frame(DWORD64 pc, char *out, size_t outn) {
    unsigned idx = (unsigned)((pc ^ (pc >> 16) ^ (pc >> 32)) & (PC_CACHE - 1));
    if (g_pcc[idx].pc == pc) { snprintf(out, outn, "%s", g_pcc[idx].name); return; }

    PSYMBOL_INFO si = (PSYMBOL_INFO)g_symbuf;
    memset(si, 0, sizeof(SYMBOL_INFO));
    si->SizeOfStruct = sizeof(SYMBOL_INFO);
    si->MaxNameLen = MAX_NAME;
    DWORD64 disp = 0;
    int have_sym = SymFromAddr(g_process, pc, &disp, si);

    DWORD64 modbase = SymGetModuleBase64(g_process, pc);
    char modname[64] = "?";
    if (modbase) {
        IMAGEHLP_MODULE64 m;
        memset(&m, 0, sizeof(m));
        m.SizeOfStruct = sizeof(m);
        if (SymGetModuleInfo64(g_process, pc, &m)) {
            const char *b = strrchr(m.ModuleName, '\\');
            if (!b) b = strrchr(m.ModuleName, '/');
            snprintf(modname, sizeof(modname), "%s", b ? b + 1 : m.ModuleName);
        }
    }

    if (have_sym)
        snprintf(out, outn, "%s!%s+0x%llx", modname, si->Name, (unsigned long long)disp);
    else
        snprintf(out, outn, "%s!0x%llx", modname, (unsigned long long)(pc - modbase));

    g_pcc[idx].pc = pc;
    snprintf(g_pcc[idx].name, sizeof(g_pcc[idx].name), "%s", out);
}

#define MAX_STACKS 8192
#define STACK_STR_LEN 2048
static char g_stacks[MAX_STACKS][STACK_STR_LEN];
static unsigned long long g_stack_n[MAX_STACKS];
static int g_nstacks;
#define STACK_BUCKETS 8192
static int g_stack_head[STACK_BUCKETS];
static int g_stack_next[MAX_STACKS];
static void record_stack(char names[MAX_STACK][MAX_NAME], int depth);

struct ThEnt { DWORD tid; HANDLE h; };
static struct ThEnt g_th[64];
static int g_nth;
static unsigned long long g_calls;

static void sample_once(void) {
    DWORD tp0 = GetTickCount();
    /* CreateToolhelp32Snapshot costs ~42ms (walks all system threads);
       refresh our handle cache every 20th round instead */
    if (g_calls % 20 == 0 || g_nth == 0) {
        for (int i = 0; i < g_nth; i++) if (g_th[i].h) CloseHandle(g_th[i].h);
        g_nth = 0;
        HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if (snap != INVALID_HANDLE_VALUE) {
            DWORD mypid = GetProcessId(g_process);
            THREADENTRY32 te; te.dwSize = sizeof(te);
            if (Thread32First(snap, &te)) {
                do {
                    if (te.th32OwnerProcessID != mypid) continue;
                    if (g_nth >= 64) break;
                    g_th[g_nth].tid = te.th32ThreadID;
                    g_th[g_nth].h = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT |
                                               THREAD_QUERY_INFORMATION, FALSE, te.th32ThreadID);
                    if (g_th[g_nth].h) g_nth++;
                } while (Thread32Next(snap, &te));
            }
            CloseHandle(snap);
        }
    }
    g_calls++;
    g_t_snap += GetTickCount() - tp0;

    struct ThEnt *th = g_th;
    int n = g_nth;

    DWORD ts0 = GetTickCount();
    int active[64];
    for (int i = 0; i < n; i++)
        active[i] = SuspendThread(th[i].h) != (DWORD)-1;
    g_t_susp += GetTickCount() - ts0;

    /* on-CPU filter: only count threads that burned CPU since the previous round */
    unsigned long long weight[64];
    int track_slot[64];
    for (int i = 0; i < n; i++) {
        weight[i] = 0;
        track_slot[i] = -1;
        if (!active[i]) continue;

        FILETIME cr, ex, kr, ut;
        if (!GetThreadTimes(th[i].h, &cr, &ex, &kr, &ut)) continue;
        unsigned long long cpu =
            (((unsigned long long)kr.dwHighDateTime << 32) | kr.dwLowDateTime) +
            (((unsigned long long)ut.dwHighDateTime << 32) | ut.dwLowDateTime);

        int ti = -1;
        for (int t = 0; t < g_nthreads_seen; t++)
            if (g_tids[t] == th[i].tid) { ti = t; break; }
        if (ti < 0 && g_nthreads_seen < MAX_THREADS) {
            ti = g_nthreads_seen++;
            g_tids[ti] = th[i].tid;
            thread_name_of(th[i].h, th[i].tid, g_tnames[ti], sizeof(g_tnames[ti]));
            g_thread_samples[ti] = 0;
            g_prev_cpu[ti] = cpu;
            g_have_prev[ti] = 1;
        }
        if (ti >= 0) {
            track_slot[i] = ti;
            if (g_have_prev[ti] && cpu > g_prev_cpu[ti])
                g_acc_cpu[ti] += cpu - g_prev_cpu[ti];
            g_prev_cpu[ti] = cpu;
            g_have_prev[ti] = 1;
            weight[i] = g_acc_cpu[ti]; /* >0 => burned CPU since last counted sample */
        }
    }

    for (int i = 0; i < n; i++) {
        if (!active[i]) continue;
        if (weight[i] == 0) continue; /* thread burned no CPU since previous round */
        CONTEXT ctx;
        memset(&ctx, 0, sizeof(ctx));
        ctx.ContextFlags = CONTEXT_FULL;
        if (!GetThreadContext(th[i].h, &ctx)) continue;

        STACKFRAME64 sf;
        memset(&sf, 0, sizeof(sf));
        sf.AddrPC.Offset    = ctx.Rip;
        sf.AddrFrame.Offset = ctx.Rbp;
        sf.AddrStack.Offset = ctx.Rsp;
        sf.AddrPC.Mode = sf.AddrFrame.Mode = sf.AddrStack.Mode = AddrModeFlat;

        char names[MAX_STACK][MAX_NAME];
        int depth = 0;
        DWORD64 last_pc = 0;

        DWORD tw0 = GetTickCount();
        while (depth < MAX_STACK &&
               StackWalk64(IMAGE_FILE_MACHINE_AMD64, g_process, th[i].h,
                           &sf, &ctx, read_mem,
                           SymFunctionTableAccess64, SymGetModuleBase64, NULL)) {
            if (sf.AddrPC.Offset == 0) break;
            if (sf.AddrPC.Offset == last_pc && depth > 0) break;
            last_pc = sf.AddrPC.Offset;
            resolve_frame(sf.AddrPC.Offset, names[depth], sizeof(names[depth]));
            depth++;
        }
        g_t_walk += GetTickCount() - tw0;

        if (depth > 0) {
            DWORD ta0 = GetTickCount();
            EnterCriticalSection(&g_lock);
            g_total_samples++;
            int ti = track_slot[i];
            if (ti >= 0) { g_thread_samples[ti]++; g_acc_cpu[ti] = 0; }
            for (int d = 0; d < depth; d++) {
                int ai = agg_find(names[d]);
                if (ai >= 0) {
                    g_agg[ai].incl++;
                    if (d == 0) g_agg[ai].self++;
                }
            }
            if (ti >= 0) {
                int c = g_tfn_cnt[ti], found = -1;
                for (int j = 0; j < c; j++)
                    if (strcmp(g_tfn_name[ti][j], names[0]) == 0) { found = j; break; }
                if (found < 0 && c < TTOP) {
                    found = c;
                    g_tfn_cnt[ti]++;
                    strncpy(g_tfn_name[ti][found], names[0], TTOP_NAME - 1);
                    g_tfn_name[ti][found][TTOP_NAME - 1] = 0;
                    g_tfn_n[ti][found] = 0;
                }
                if (found >= 0) g_tfn_n[ti][found]++;
            }
            record_stack(names, depth);
            LeaveCriticalSection(&g_lock);
            g_t_agg += GetTickCount() - ta0;
        }
    }

    for (int i = 0; i < n; i++) {
        if (active[i]) ResumeThread(th[i].h);
        /* handles stay cached until next refresh */
    }
}

static int cmp_agg(const void *a, const void *b) {
    const SymAgg *x = a, *y = b;
    if (x->self < y->self) return 1;
    if (x->self > y->self) return -1;
    return 0;
}

/* ---- full-stack aggregation ---- */
#define MAX_STACKS 8192
#define STACK_STR_LEN 2048
static char g_stacks[MAX_STACKS][STACK_STR_LEN];
static unsigned long long g_stack_n[MAX_STACKS];
static int g_nstacks;

static void record_stack(char names[MAX_STACK][MAX_NAME], int depth) {
    char buf[STACK_STR_LEN];
    size_t off = 0;
    for (int d = 0; d < depth && off < sizeof(buf) - 2; d++) {
        if (d) { buf[off++] = '<'; buf[off++] = '-'; }
        size_t l = strlen(names[d]);
        if (off + l >= sizeof(buf) - 1) break;
        memcpy(buf + off, names[d], l);
        off += l;
    }
    buf[off] = 0;

    /* FNV-1a buckets: linear strcmp over all stacks was ~50ms/sample */
    unsigned long long h = 1469598103934665603ULL;
    for (size_t i = 0; i < off; i++) { h ^= (unsigned char)buf[i]; h *= 1099511628211ULL; }
    int b = (int)(h & (STACK_BUCKETS - 1));
    for (int i = g_stack_head[b]; i >= 0; i = g_stack_next[i]) {
        if (strcmp(g_stacks[i], buf) == 0) { g_stack_n[i]++; return; }
    }
    if (g_nstacks >= MAX_STACKS) return;
    int s = g_nstacks++;
    strncpy(g_stacks[s], buf, STACK_STR_LEN - 1);
    g_stacks[s][STACK_STR_LEN - 1] = 0;
    g_stack_n[s] = 1;
    g_stack_next[s] = g_stack_head[b];
    g_stack_head[b] = s;
}

static int cmp_stack(const void *a, const void *b) {
    int ia = *(const int *)a, ib = *(const int *)b;
    if (g_stack_n[ia] < g_stack_n[ib]) return 1;
    if (g_stack_n[ia] > g_stack_n[ib]) return -1;
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 5) {
        fprintf(stderr, "usage: profiler_cli <pid> <seconds> <interval_ms> <out.txt> [sym_path]\n");
        return 2;
    }
    DWORD pid = (DWORD)strtoul(argv[1], NULL, 10);
    int seconds = atoi(argv[2]);
    g_interval_ms = (DWORD)strtoul(argv[3], NULL, 10);
    const char *outpath = argv[4];
    const char *sympath = argc > 5 ? argv[5] : NULL;

    InitializeCriticalSection(&g_lock);
    for (int i = 0; i < STACK_BUCKETS; i++) g_stack_head[i] = -1;

    g_process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | SYNCHRONIZE,
                            FALSE, pid);
    if (!g_process) {
        fprintf(stderr, "OpenProcess failed: %lu\n", GetLastError());
        return 1;
    }

    SymSetOptions(SYMOPT_LOAD_LINES | SYMOPT_UNDNAME |
                  SYMOPT_FAIL_CRITICAL_ERRORS | SYMOPT_NO_PROMPTS);
    if (!SymInitialize(g_process, sympath, FALSE))
        fprintf(stderr, "SymInitialize failed: %lu (continuing)\n", GetLastError());

    /* preload all modules so SymGetModuleBase/Info work */
    {
        HMODULE mods[512];
        DWORD need = 0;
        int loaded = 0, failed = 0;
        DWORD first_err = 0;
        if (EnumProcessModules(g_process, mods, sizeof(mods), &need)) {
            for (DWORD k = 0; k < need / sizeof(HMODULE); k++) {
                char path[MAX_PATH] = "";
                GetModuleFileNameExA(g_process, mods[k], path, sizeof(path));
                DWORD64 r = SymLoadModuleEx(g_process, NULL, path, NULL,
                                            (DWORD64)(uintptr_t)mods[k], 0, NULL, 0);
                if (r) loaded++;
                else {
                    DWORD e = GetLastError();
                    if (!first_err) first_err = e;
                    failed++;
                }
            }
        }
        fprintf(stderr, "modules: enum=%lu loaded=%d failed=%d first_err=%lu\n",
                (unsigned long)(need / sizeof(HMODULE)), loaded, failed, first_err);
        char exepath[MAX_PATH] = "";
        GetModuleFileNameExA(g_process, NULL, exepath, sizeof(exepath));
        fprintf(stderr, "exe: %s\n", exepath);
    }

    fprintf(stderr, "sampling pid=%lu for %ds every %ums...\n",
            pid, seconds, g_interval_ms);
    timeBeginPeriod(1);
    DWORD start = GetTickCount();
    unsigned long long rounds = 0;
    unsigned long long sum_round = 0, max_round = 0;
    while ((int)(GetTickCount() - start) < seconds * 1000) {
        if (WaitForSingleObject(g_process, 0) == WAIT_OBJECT_0) {
            fprintf(stderr, "target exited after %lus\n",
                    (unsigned long)((GetTickCount() - start) / 1000));
            break;
        }
        DWORD rt0 = GetTickCount();
        sample_once();
        DWORD rdt = GetTickCount() - rt0;
        sum_round += rdt;
        if (rdt > max_round) max_round = rdt;
        rounds++;
        if (rounds % 100 == 0)
            fprintf(stderr, "round %llu: sample_once last=%lums avg=%.1f max=%lu (snap=%llu susp=%llu walk=%llu agg=%llu ms)\n",
                    (unsigned long long)rounds, (unsigned long)rdt,
                    sum_round / (double)rounds, (unsigned long)max_round,
                    (unsigned long long)g_t_snap, (unsigned long long)g_t_susp,
                    (unsigned long long)g_t_walk, (unsigned long long)g_t_agg);
        Sleep(g_interval_ms);
    }
    DWORD elapsed_ms = GetTickCount() - start;
    timeEndPeriod(1);

    FILE *f = fopen(outpath, "w");
    if (!f) { fprintf(stderr, "cannot open %s\n", outpath); return 1; }

    fprintf(f, "== profiler_cli report ==\n");
    fprintf(f, "pid=%lu cfg_seconds=%d elapsed_ms=%lu rounds=%llu interval_ms=%lu total_samples=%llu\n\n",
            pid, seconds, (unsigned long)elapsed_ms,
            (unsigned long long)rounds, (unsigned long)g_interval_ms,
            (unsigned long long)g_total_samples);

    qsort(g_agg, (size_t)g_nagg, sizeof(g_agg[0]), cmp_agg);

    fprintf(f, "== top self (self%% >= 0.05) ==\n");
    fprintf(f, "%8s %8s %8s  %s\n", "self%", "incl%", "self_n", "function");
    for (int i = 0; i < g_nagg; i++) {
        double sp = g_total_samples ? 100.0 * g_agg[i].self / (double)g_total_samples : 0;
        double ip = g_total_samples ? 100.0 * g_agg[i].incl / (double)g_total_samples : 0;
        if (sp < 0.05) break;
        fprintf(f, "%7.2f%% %7.2f%% %8llu  %s\n", sp, ip,
                (unsigned long long)g_agg[i].self, g_agg[i].key);
    }

    fprintf(f, "\n== threads (sorted by thread cpu time) ==\n");
    /* refresh cpu times of live threads; g_prev_cpu holds last-seen value for dead ones */
    for (int t = 0; t < g_nthreads_seen; t++) {
        HANDLE ht = OpenThread(THREAD_QUERY_INFORMATION, FALSE, g_tids[t]);
        if (ht) {
            FILETIME cr, ex, kr, ut;
            if (GetThreadTimes(ht, &cr, &ex, &kr, &ut)) {
                unsigned long long cpu =
                    (((unsigned long long)kr.dwHighDateTime << 32) | kr.dwLowDateTime) +
                    (((unsigned long long)ut.dwHighDateTime << 32) | ut.dwLowDateTime);
                if (cpu > g_prev_cpu[t]) g_prev_cpu[t] = cpu;
            }
            CloseHandle(ht);
        }
    }
    unsigned long long total_cpu = 0;
    for (int t = 0; t < g_nthreads_seen; t++) total_cpu += g_prev_cpu[t];

    int torder[MAX_THREADS];
    int tcnt = 0;
    for (int t = 0; t < g_nthreads_seen; t++) {
        if (g_thread_samples[t] == 0 && g_prev_cpu[t] == 0) continue;
        torder[tcnt++] = t;
    }
    for (int a = 1; a < tcnt; a++) {  /* insertion sort by cpu desc */
        int key = torder[a], b = a - 1;
        while (b >= 0 && g_prev_cpu[torder[b]] < g_prev_cpu[key]) { torder[b + 1] = torder[b]; b--; }
        torder[b + 1] = key;
    }
    fprintf(f, "%10s %8s %9s %9s  %s\n", "cpu_ms", "cpu%", "samples", "samp%", "thread");
    for (int k = 0; k < tcnt; k++) {
        int t = torder[k];
        double cp = total_cpu ? 100.0 * g_prev_cpu[t] / (double)total_cpu : 0;
        double p = g_total_samples ? 100.0 * g_thread_samples[t] / (double)g_total_samples : 0;
        fprintf(f, "%10.1f %7.2f%% %9llu %7.2f%%  %-24s (tid %lu)\n",
                g_prev_cpu[t] / 10000.0, cp,
                (unsigned long long)g_thread_samples[t], p, g_tnames[t], g_tids[t]);
        /* top-5 self functions of this thread */
        for (int pass = 0; pass < 5 && g_tfn_cnt[t] > 0; pass++) {
            int bi = -1;
            for (int j = 0; j < g_tfn_cnt[t]; j++)
                if (g_tfn_n[t][j] && (bi < 0 || g_tfn_n[t][j] > g_tfn_n[t][bi])) bi = j;
            if (bi < 0) break;
            double tp = g_thread_samples[t] ? 100.0 * g_tfn_n[t][bi] / (double)g_thread_samples[t] : 0;
            fprintf(f, "        %5.1f%% of samples  %s\n", tp, g_tfn_name[t][bi]);
            g_tfn_n[t][bi] = 0;
        }
    }

    int order[MAX_STACKS];
    for (int i = 0; i < g_nstacks; i++) order[i] = i;
    qsort(order, (size_t)g_nstacks, sizeof(int), cmp_stack);
    fprintf(f, "\n== top stacks (top 40) ==\n");
    for (int i = 0; i < g_nstacks && i < 40; i++) {
        int si = order[i];
        double p = g_total_samples ? 100.0 * g_stack_n[si] / (double)g_total_samples : 0;
        fprintf(f, "\n[%5.2f%%] %s\n", p, g_stacks[si]);
    }

    fclose(f);
    SymCleanup(g_process);
    CloseHandle(g_process);
    fprintf(stderr, "done: %s (samples=%llu)\n", outpath,
            (unsigned long long)g_total_samples);
    return 0;
}
