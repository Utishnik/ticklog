#include <stdio.h>
#include <stdint.h>

static inline uint64_t rdtsc_raw(void) {
    unsigned a, d;
    __asm__ volatile("rdtsc" : "=a"(a), "=d"(d));
    return ((uint64_t)d << 32) | a;
}

static inline uint64_t rdtsc_lfence(void) {
    unsigned a, d;
    __asm__ volatile("lfence; rdtsc" : "=a"(a), "=d"(d));
    return ((uint64_t)d << 32) | a;
}

int main(void) {
    volatile uint64_t sink = 0;
    uint64_t t0, t1;
    long N = 100000000;

    for (long i = 0; i < 1000000; i++) sink += rdtsc_raw();

    // dependent chain: each rdtsc feeds the accumulator used next
    uint64_t acc = 0;
    t0 = rdtsc_raw();
    for (long i = 0; i < N; i++) {
        acc += rdtsc_raw() + acc;
    }
    t1 = rdtsc_raw();
    printf("dependent chain: %.2f cycles/iter\n", (double)(t1 - t0) / N);
    sink += acc;

    // 4 independent accumulators: lets rdtsc results pipeline
    uint64_t a0 = 0, a1 = 0, a2 = 0, a3 = 0;
    t0 = rdtsc_raw();
    for (long i = 0; i < N; i += 4) {
        a0 += rdtsc_raw();
        a1 += rdtsc_raw();
        a2 += rdtsc_raw();
        a3 += rdtsc_raw();
    }
    t1 = rdtsc_raw();
    printf("4-way unrolled:  %.2f cycles/iter\n", (double)(t1 - t0) / (N / 4) / 4);
    sink += a0 + a1 + a2 + a3;

    // lfence variant (serialized, worst case for a "safe" timestamp)
    uint64_t b = 0;
    t0 = rdtsc_raw();
    for (long i = 0; i < N; i++) {
        b += rdtsc_lfence();
    }
    t1 = rdtsc_raw();
    printf("lfence+rdtsc:    %.2f cycles/iter\n", (double)(t1 - t0) / N);
    sink += b;

    printf("sink=%lu\n", (unsigned long)sink);
    return 0;
}
