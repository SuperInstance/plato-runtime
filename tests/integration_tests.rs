use plato_runtime::{
    config::RuntimeConfig,
    discovery::{cpu::discover_cpu, memory::discover_memory, topology::detect_cache_topology},
    gpu::{
        cuda::is_cuda_available,
        kernel_cache::{CachedKernel, KernelCache},
    },
    schedule::work_stealing::WorkStealingDeque,
    types::{Capabilities, Priority, Task},
    AdaptiveScheduler,
};
use std::time::Instant;

// ─── 1. CPU discovery completes without panic ────────────────────────────────

#[test]
fn test_discover_cpu_no_panic() {
    let res = discover_cpu();
    // ResourceId must be non-empty — the function always sets one
    assert!(!res.id.0.is_empty(), "CPU resource id should not be empty");
}

// ─── 2. Memory detection returns positive values ─────────────────────────────

#[test]
fn test_discover_memory_positive() {
    let mut caps = Capabilities::default();
    discover_memory(&mut caps);
    // On any real Linux system /proc/meminfo must report > 0 bytes
    assert!(caps.memory_total > 0, "memory_total should be positive, got {}", caps.memory_total);
    assert!(
        caps.memory_available <= caps.memory_total,
        "available ({}) should not exceed total ({})",
        caps.memory_available,
        caps.memory_total
    );
}

// ─── 3. Cache topology returns reasonable values ──────────────────────────────

#[test]
fn test_detect_cache_topology_reasonable() {
    let info = detect_cache_topology();
    // Even the hardcoded defaults are sane; sysfs values should not be zero
    assert!(info.l1_data > 0, "L1d must be > 0");
    assert!(info.l1_instruction > 0, "L1i must be > 0");
    assert!(info.l2 > 0, "L2 must be > 0");
    assert!(info.l3 > 0, "L3 must be > 0");
    // Cache line size must be a power of two between 32 and 256 bytes
    assert!(info.cache_line >= 32, "cache_line {} < 32", info.cache_line);
    assert!(info.cache_line <= 256, "cache_line {} > 256", info.cache_line);
    assert!(
        info.cache_line.is_power_of_two(),
        "cache_line {} is not a power of two",
        info.cache_line
    );
}

// ─── 4. AdaptiveScheduler submit + poll roundtrip ────────────────────────────

#[test]
fn test_scheduler_submit_poll_roundtrip() {
    let sched = AdaptiveScheduler::new();
    assert_eq!(sched.queue_depth(), 0);

    let _handle = sched.submit(Task::Benchmark { duration_ms: 1 }, Priority::Normal);
    assert_eq!(sched.queue_depth(), 1);

    let result = sched.poll();
    assert!(result.is_some(), "poll should return the submitted task");
    assert_eq!(sched.queue_depth(), 0);

    // Empty queue returns None
    assert!(sched.poll().is_none());
}

// ─── 5. AdaptiveScheduler priority ordering ──────────────────────────────────

#[test]
fn test_scheduler_priority_ordering() {
    let sched = AdaptiveScheduler::new();

    // Submit lower-priority tasks first, then a Critical one
    sched.submit(Task::Benchmark { duration_ms: 1 }, Priority::Normal);
    sched.submit(Task::Benchmark { duration_ms: 1 }, Priority::Low);
    sched.submit(Task::Benchmark { duration_ms: 1 }, Priority::Critical);
    sched.submit(Task::Benchmark { duration_ms: 1 }, Priority::High);

    // First poll must return the Critical task
    let (_, prio) = sched.poll().expect("should have a task");
    assert_eq!(prio, Priority::Critical, "first task polled must be Critical");

    // Second must be High
    let (_, prio) = sched.poll().expect("should have a task");
    assert_eq!(prio, Priority::High, "second task polled must be High");

    // Third must be Normal
    let (_, prio) = sched.poll().expect("should have a task");
    assert_eq!(prio, Priority::Normal, "third task polled must be Normal");

    // Fourth must be Low
    let (_, prio) = sched.poll().expect("should have a task");
    assert_eq!(prio, Priority::Low, "fourth task polled must be Low");

    assert!(sched.poll().is_none());
}

// ─── 6. WorkStealingDeque push / pop / steal ─────────────────────────────────

#[test]
fn test_work_stealing_deque() {
    let deque: WorkStealingDeque<u32> = WorkStealingDeque::new();

    assert!(deque.is_empty());
    assert_eq!(deque.len(), 0);

    deque.push(1);
    deque.push(2);
    deque.push(3);
    assert_eq!(deque.len(), 3);

    // pop() is LIFO (owner, from the bottom)
    assert_eq!(deque.pop(), Some(3));
    assert_eq!(deque.pop(), Some(2));

    // steal() is FIFO (thief, from the top)
    deque.push(10);
    deque.push(20);
    assert_eq!(deque.steal(), Some(1));  // oldest item
    assert_eq!(deque.steal(), Some(10));
    assert_eq!(deque.steal(), Some(20));

    assert!(deque.is_empty());
    assert_eq!(deque.pop(), None);
    assert_eq!(deque.steal(), None);
}

// ─── 7. KernelCache hit / miss stats ─────────────────────────────────────────

#[test]
fn test_kernel_cache_hit_miss_stats() {
    let mut cache = KernelCache::new();

    // Misses on empty cache
    assert!(cache.get("kernel_a").is_none());
    assert!(cache.get("kernel_b").is_none());

    let (hits, misses, count) = cache.stats();
    assert_eq!(hits, 0);
    assert_eq!(misses, 2);
    assert_eq!(count, 0);

    // Insert a kernel
    cache.insert(CachedKernel {
        name: "kernel_a".into(),
        source_hash: 0xDEADBEEF,
        compiled_at: Instant::now(),
        compile_time_ms: 42,
        launch_count: 0,
    });

    // Now a hit
    assert!(cache.get("kernel_a").is_some());
    // Still a miss for the unknown one
    assert!(cache.get("kernel_z").is_none());

    let (hits, misses, count) = cache.stats();
    assert_eq!(hits, 1, "expected 1 hit");
    assert_eq!(misses, 3, "expected 3 misses total");
    assert_eq!(count, 1, "expected 1 cached kernel");
}

// ─── 8. Config default values ────────────────────────────────────────────────

#[test]
fn test_config_default_values() {
    let cfg = RuntimeConfig::default();

    assert_eq!(cfg.max_threads, 0, "max_threads 0 means auto-detect");
    assert!(cfg.gpu_enabled, "GPU should be enabled by default");
    assert_eq!(cfg.idle_threshold_ms, 30_000, "idle threshold should be 30 s");
    assert!(
        cfg.idle_budget > 0.0 && cfg.idle_budget <= 1.0,
        "idle_budget out of range: {}",
        cfg.idle_budget
    );
    assert!(
        cfg.active_budget > 0.0 && cfg.active_budget <= 1.0,
        "active_budget out of range: {}",
        cfg.active_budget
    );
    assert!(cfg.active_budget < cfg.idle_budget, "idle budget should exceed active budget");
    assert!(cfg.auto_profile, "auto_profile should default to true");
    assert!(cfg.plato_url.is_none(), "plato_url should default to None");
}

// ─── 9. TSC frequency estimation returns non-zero on x86_64 ──────────────────

#[test]
fn test_tsc_frequency_nonzero() {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::_rdtsc;

        let wall_start = Instant::now();
        let tsc_start = unsafe { _rdtsc() };

        // Sleep for a short interval so we get a meaningful delta
        std::thread::sleep(std::time::Duration::from_millis(20));

        let tsc_end = unsafe { _rdtsc() };
        let elapsed = wall_start.elapsed().as_secs_f64();

        let tsc_delta = tsc_end.wrapping_sub(tsc_start) as f64;
        let estimated_hz = tsc_delta / elapsed;

        assert!(
            estimated_hz > 0.0,
            "TSC frequency estimate must be non-zero on x86_64, got {}",
            estimated_hz
        );
        // Sanity: expect at least 100 MHz, at most 10 GHz
        assert!(
            estimated_hz > 100_000_000.0,
            "TSC freq suspiciously low: {:.2} MHz",
            estimated_hz / 1e6
        );
        assert!(
            estimated_hz < 10_000_000_000.0,
            "TSC freq suspiciously high: {:.2} GHz",
            estimated_hz / 1e9
        );
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        // On non-x86_64 the runtime returns 0.0 — that is the documented behaviour
        println!("Skipping TSC test: not on x86_64");
    }
}

// ─── 10. CUDA detection doesn't panic ────────────────────────────────────────

#[test]
fn test_cuda_detection_no_panic() {
    // The result (true/false) depends on the machine; we only verify no panic
    let _available = is_cuda_available();
}
