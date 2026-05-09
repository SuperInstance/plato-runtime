/// Benchmark tasks for self-optimization

use std::time::Instant;
use crate::task::constraint;

/// Run a comprehensive benchmark suite
pub struct BenchmarkResult {
    pub constraint_i32_ops: f64,
    pub constraint_i32_time_ms: f64,
    pub eisenstein_ops: f64,
    pub eisenstein_time_ms: f64,
}

/// Run benchmark with varying sizes to find optimal batch size
pub fn run_sizing_benchmark() -> Vec<(usize, f64)> {
    let sizes = [1_000, 10_000, 100_000, 1_000_000];
    let mut results = Vec::new();
    
    for &size in &sizes {
        let start = Instant::now();
        let iterations = (10_000_000 / size).max(1);
        for _ in 0..iterations {
            let _ = constraint::bench_constraint_i32(size);
        }
        let elapsed = start.elapsed().as_secs_f64();
        let mops = (size as f64 * iterations as f64) / elapsed / 1e6;
        results.push((size, mops));
    }
    
    results
}

pub fn print_sizing_results(results: &[(usize, f64)]) {
    println!("  Batch Size | Throughput (Mops/s)");
    println!("  -----------+--------------------");
    for (size, mops) in results {
        println!("  {:>10} | {:.2}", size, mops);
    }
}
