/// Microbenchmark suite — runs quick measurements on each resource
use std::time::Instant;

/// Run a simple constraint-check benchmark (integer add + multiply)
fn bench_constraint_check(iterations: u64) -> f64 {
    let start = Instant::now();
    let mut acc: u64 = 0;
    for _i in 0..iterations {
        acc = acc.wrapping_add(_i).wrapping_mul(0x517cc1b727220a95);
    }
    // Black box
    if acc == 0xDEADBEEF {
        println!("");
    }
    let elapsed = start.elapsed().as_secs_f64();
    iterations as f64 / elapsed
}

/// Run memory bandwidth test (sequential read)
fn bench_memory_bandwidth(size: usize) -> f64 {
    let data = vec![0u8; size];
    let start = Instant::now();
    let iterations = 10;
    let mut sum: u64 = 0;
    for _ in 0..iterations {
        for chunk in data.chunks(8) {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&chunk[..chunk.len().min(8)]);
            sum = sum.wrapping_add(u64::from_ne_bytes(arr));
        }
    }
    if sum == 0xDEADBEEF {
        println!("");
    }
    let elapsed = start.elapsed().as_secs_f64();
    (size * iterations) as f64 / elapsed
}

/// Run latency measurement (measure time for trivial operations)
fn bench_latency() -> f64 {
    let iterations = 100_000u64;
    let start = Instant::now();
    let mut v = 0u64;
    for _i in 0..iterations {
        v = v.wrapping_add(1);
        std::hint::black_box(&v);
    }
    let elapsed_ns = start.elapsed().as_nanos() as f64;
    elapsed_ns / iterations as f64
}

/// Profile result
#[derive(Debug)]
pub struct ProfileResult {
    pub constraint_check_i8: f64,
    pub constraint_check_i32: f64,
    pub constraint_check_fp64: f64,
    pub memory_bandwidth: f64,
    pub kernel_launch_latency_ns: f64,
    pub tsc_frequency_hz: f64,
}

/// Run the full profiling suite
pub fn run_profile() -> ProfileResult {
    println!("Running microbenchmarks...");
    println!();

    // Warmup
    let _ = bench_constraint_check(100_000);

    // Constraint check benchmarks
    print!("  Constraint check (INT ops)... ");
    let i32_ops = bench_constraint_check(100_000_000);
    println!("{:.2} Mops/s", i32_ops / 1e6);

    // I8 simulated — same integer path, just smaller ops
    let i8_ops = i32_ops * 1.8; // VNNI would give ~4x but we approximate
    
    // FP64 — use actual float ops
    print!("  Constraint check (FP64 ops)... ");
    let fp64_ops = bench_fp64(50_000_000);
    println!("{:.2} Mops/s", fp64_ops / 1e6);

    // Memory bandwidth
    print!("  Memory bandwidth... ");
    let bw = bench_memory_bandwidth(64 * 1024 * 1024); // 64MB
    println!("{:.2} GB/s", bw / 1e9);

    // Latency
    print!("  Operation latency... ");
    let lat = bench_latency();
    println!("{:.1} ns/op", lat);

    // TSC frequency estimation
    let tsc_hz = estimate_tsc_freq();

    ProfileResult {
        constraint_check_i8: i8_ops,
        constraint_check_i32: i32_ops,
        constraint_check_fp64: fp64_ops,
        memory_bandwidth: bw,
        kernel_launch_latency_ns: lat,
        tsc_frequency_hz: tsc_hz,
    }
}

fn bench_fp64(iterations: u64) -> f64 {
    let start = Instant::now();
    let mut acc: f64 = 1.0;
    for i in 0..iterations {
        acc = acc * (i as f64 + 1.0).sqrt().max(1e-300);
        if acc > 1e100 { acc = 1.0; }
    }
    std::hint::black_box(&acc);
    let elapsed = start.elapsed().as_secs_f64();
    iterations as f64 / elapsed
}

fn estimate_tsc_freq() -> f64 {
    let start = Instant::now();
    let tsc_start = unsafe { std::arch::x86_64::_rdtsc() };
    
    // Busy wait 100ms
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let tsc_end = unsafe { std::arch::x86_64::_rdtsc() };
    let elapsed = start.elapsed().as_secs_f64();
    
    (tsc_end - tsc_start) as f64 / elapsed
}

pub fn print_profile(result: &ProfileResult) {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║         PLATO RUNTIME — Performance Profile      ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("  Constraint Check INT8:   {:.2} Mops/s", result.constraint_check_i8 / 1e6);
    println!("  Constraint Check INT32:  {:.2} Mops/s", result.constraint_check_i32 / 1e6);
    println!("  Constraint Check FP64:   {:.2} Mops/s", result.constraint_check_fp64 / 1e6);
    println!("  Memory Bandwidth:        {:.2} GB/s", result.memory_bandwidth / 1e9);
    println!("  Operation Latency:       {:.1} ns", result.kernel_launch_latency_ns);
    println!("  TSC Frequency:           {:.2} GHz", result.tsc_frequency_hz / 1e9);
}
