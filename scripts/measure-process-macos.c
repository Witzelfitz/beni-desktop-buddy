// Manual, per-process measurement. Not linked into Beni and never run by CI.
// Apple uses Mach ticks for ri_user_time/ri_system_time; see XNU's
// tests/recount/recount_perf_tests.c.
#ifndef __APPLE__
#error This measurement utility requires macOS.
#endif
#include <errno.h>
#include <limits.h>
#include <libproc.h>
#include <mach/mach_time.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/resource.h>
#include <time.h>

static int positive_integer(const char *text, long maximum, int *value) {
    char *end;
    errno = 0;
    long parsed = strtol(text, &end, 10);
    if (errno || end == text || *end || parsed < 1 || parsed > maximum) {
        return 0;
    }
    *value = (int)parsed;
    return 1;
}

int main(int argc, char **argv) {
    int pid, seconds, samples;
    if (argc != 4 || !positive_integer(argv[1], INT_MAX, &pid) ||
        !positive_integer(argv[2], 60, &seconds) ||
        !positive_integer(argv[3], 60, &samples)) {
        fprintf(stderr, "Usage: %s PID seconds(1..60) samples(1..60)\n", argv[0]);
        return 2;
    }
    mach_timebase_info_data_t timebase;
    if (mach_timebase_info(&timebase) != KERN_SUCCESS || !timebase.denom) {
        fprintf(stderr, "Mach timebase unavailable\n");
        return 1;
    }
    double ns_per_tick = (double)timebase.numer / timebase.denom;
    puts("sample,seconds,cpu_pct_one_core,wakeups_per_second,footprint_mib");
    fflush(stdout);
    for (int sample = 1; sample <= samples; sample++) {
        struct rusage_info_v4 before = {0}, after = {0};
        if (proc_pid_rusage(pid, RUSAGE_INFO_V4, (rusage_info_t *)&before)) {
            perror("proc_pid_rusage");
            return 1;
        }
        uint64_t begin = mach_continuous_time();
        struct timespec remaining = {.tv_sec = seconds, .tv_nsec = 0};
        while (nanosleep(&remaining, &remaining)) {
            if (errno != EINTR) {
                perror("nanosleep");
                return 1;
            }
        }
        uint64_t end = mach_continuous_time();
        if (proc_pid_rusage(pid, RUSAGE_INFO_V4, (rusage_info_t *)&after)) {
            perror("proc_pid_rusage");
            return 1;
        }
        double elapsed_ns = (double)(end - begin) * ns_per_tick;
        if (before.ri_proc_start_abstime != after.ri_proc_start_abstime ||
            after.ri_user_time < before.ri_user_time ||
            after.ri_system_time < before.ri_system_time ||
            after.ri_interrupt_wkups < before.ri_interrupt_wkups ||
            elapsed_ns > ((double)seconds + 1) * 1e9) {
            fprintf(stderr, "Process changed or device slept; discard this run\n");
            return 1;
        }
        double cpu_ns = ((double)(after.ri_user_time - before.ri_user_time) +
                         (double)(after.ri_system_time - before.ri_system_time)) * ns_per_tick;
        printf("%d,%.3f,%.4f,%.3f,%.2f\n", sample, elapsed_ns / 1e9,
               100 * cpu_ns / elapsed_ns,
               (double)(after.ri_interrupt_wkups - before.ri_interrupt_wkups) / (elapsed_ns / 1e9),
               (double)after.ri_phys_footprint / 1048576);
        fflush(stdout);
    }
    return 0;
}
