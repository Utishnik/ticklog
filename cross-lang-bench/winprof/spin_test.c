#define _WIN32_WINNT 0x0601
#include <windows.h>
#include <stdio.h>

static volatile int g_run = 1;
static DWORD WINAPI spin(LPVOID p) {
    (void)p;
    unsigned long long x = 1;
    while (g_run) x = x * 6364136223846793005ULL + 1;
    return (DWORD)(x & 0);
}

int main(void) {
    HANDLE t = CreateThread(NULL, 0, spin, NULL, 0, NULL);
    Sleep(200);
    HANDLE h = t; /* same thread */
    FILETIME cr, ex, kr, ut;
    if (!GetThreadTimes(h, &cr, &ex, &kr, &ut)) { printf("first GetThreadTimes failed %lu\n", GetLastError()); return 1; }
    unsigned long long prev = (((unsigned long long)kr.dwHighDateTime << 32) | kr.dwLowDateTime)
                            + (((unsigned long long)ut.dwHighDateTime << 32) | ut.dwLowDateTime);
    int polls = 0, nonzero = 0;
    unsigned long long maxd = 0, total = 0;
    DWORD start = GetTickCount();
    while (GetTickCount() - start < 3000) {
        Sleep(5);
        GetThreadTimes(h, &cr, &ex, &kr, &ut);
        unsigned long long cpu = (((unsigned long long)kr.dwHighDateTime << 32) | kr.dwLowDateTime)
                               + (((unsigned long long)ut.dwHighDateTime << 32) | ut.dwLowDateTime);
        unsigned long long d = cpu - prev;
        polls++;
        if (d) { nonzero++; total += d; if (d > maxd) maxd = d; }
        prev = cpu;
    }
    g_run = 0;
    WaitForSingleObject(t, INFINITE);
    GetThreadTimes(t, &cr, &ex, &kr, &ut);
    unsigned long long final_cpu = (((unsigned long long)kr.dwHighDateTime << 32) | kr.dwLowDateTime)
                                 + (((unsigned long long)ut.dwHighDateTime << 32) | ut.dwLowDateTime);
    printf("polls=%d nonzero=%d (%.1f%%) max_delta_ms=%.1f sum_delta_ms=%.1f final_cpu_ms=%.1f\n",
           polls, nonzero, 100.0 * nonzero / polls, maxd / 10000.0, total / 10000.0,
           final_cpu / 10000.0);
    return 0;
}
