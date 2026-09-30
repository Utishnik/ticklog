#include <stdio.h>
#include <stdint.h>

static inline uint64_t rdtsc_raw(void) {
    unsigned a, d;
    __asm__ volatile("rdtsc" : "=a"(a), "=d"(d));
    return ((uint64_t)d << 32) | a;
}

int main(void) {
    volatile uint64_t sink = 0;
    uint64_t t0, t1;
    long N = 100000000;

    for (long i = 0; i < 1000000; i++) sink += rdtsc_raw();

    t0 = rdtsc_raw();
    for (long i = 0; i < N; i++) sink += rdtsc_raw();
    t1 = rdtsc_raw();
    printf("rdtsc loop: %.2f cycles/iter (sink=%lu)\n",
           (double)(t1 - t0) / N, (unsigned long)sink);

    uint64_t buf[8] = {0};
    t0 = rdtsc_raw();
    for (long i = 0; i < N; i++) {
        unsigned a, d;
        __asm__ volatile("rdtsc" : "=a"(a), "=d"(d));
        buf[i & 7] = ((uint64_t)d << 32) | a;
    }
    t1 = rdtsc_raw();
    printf("timestamp+store: %.2f cycles/iter\n", (double)(t1 - t0) / N);
    return 0;
}
