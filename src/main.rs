use plato_runtime::{PlatoRuntime, RuntimeConfig};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "discover" => cmd_discover(),
        "profile" => cmd_profile(),
        "monitor" => cmd_monitor(),
        "optimize" => cmd_optimize(),
        "benchmark" => cmd_benchmark(),
        "idle-harvest" => cmd_idle_harvest(),
        "help" | "--help" | "-h" => cmd_help(),
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'plato-runtime help' for usage");
            std::process::exit(1);
        }
    }
}

fn cmd_help() {
    println!("PLATO Runtime — Self-discovering compute runtime");
    println!();
    println!("Usage: plato-runtime <command>");
    println!();
    println!("Commands:");
    println!("  discover      Show all discovered resources + capabilities");
    println!("  profile       Run full profiling suite");
    println!("  monitor       Live dashboard (refresh every 1s)");
    println!("  optimize      Run self-optimization cycle");
    println!("  benchmark     Run constraint benchmarks");
    println!("  idle-harvest  Start idle harvester daemon");
    println!("  help          Show this help");
}

fn cmd_discover() {
    let rt = PlatoRuntime::with_config(RuntimeConfig::default())
        .expect("Failed to initialize runtime");
    
    discovery::print_discovery(&rt.resources);
    println!();
    
    // Show scheduler info
    println!("── Scheduler ─────────────────────────────────────");
    schedule::print_affinity_info();
    println!();
    
    // Show GPU details
    if let Some(_gpu_res) = rt.resources.gpu_resources().first() {
        println!("── GPU ───────────────────────────────────────────");
        if let Some(info) = gpu::cuda::get_device_info() {
            gpu::cuda::print_cuda_info(&info);
        }
        println!();
    }
}

fn cmd_profile() {
    let result = profile::run_profile();
    println!();
    profile::print_profile(&result);
}

fn cmd_monitor() {
    println!("PLATO Runtime Monitor — Ctrl+C to stop");
    println!();
    
    let mut rt = PlatoRuntime::with_config(RuntimeConfig {
        auto_profile: false,
        ..Default::default()
    }).expect("Failed to initialize runtime");
    
    // Cache CPU info as strings (no borrow issues)
    let cpu_info: (String, Option<u32>, Option<u32>) = {
        let res = rt.resources();
        let cpus = res.cpu_resources();
        let cpu = cpus.first();
        cpu.map(|c| (c.id.to_string(), c.capabilities.cores, c.capabilities.threads))
            .unwrap_or_default()
    };
    
    loop {
        // Clear and redraw
        print!("\x1B[2J\x1B[H");
        
        println!("╔══════════════════════════════════════════════════╗");
        println!("║         PLATO RUNTIME — Live Monitor             ║");
        println!("╚══════════════════════════════════════════════════╝");
        println!();
        
        // Uptime
        println!("  Uptime: {:.0}s", rt.uptime().as_secs_f64());
        println!();
        
        // CPU
        {
            println!("── CPU ───────────────────────────────────────────");
            println!("  {}", cpu_info.0);
            if let Some(cores) = cpu_info.1 {
                println!("  Cores: {} / Threads: {}", cores, cpu_info.2.unwrap_or(0));
            }
        }
        
        // Memory (refresh)
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            println!();
            println!("── Memory ────────────────────────────────────────");
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") || line.starts_with("MemAvailable:") || line.starts_with("SwapTotal:") || line.starts_with("SwapFree:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let kb: f64 = parts[1].parse().unwrap_or(0.0);
                        println!("  {}: {:.1} GB", parts[0].trim_end_matches(':'), kb / 1_048_576.0);
                    }
                }
            }
        }
        
        // Load
        if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
            println!();
            println!("── System Load ───────────────────────────────────");
            println!("  {}", loadavg.trim());
        }
        
        // Idle state
        println!();
        println!("── Idle Harvester ────────────────────────────────");
        rt.tick();
        rt.idle_harvester.print_status();
        
        println!();
        println!("  Press Ctrl+C to stop");
        
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn cmd_optimize() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║       PLATO RUNTIME — Self-Optimization          ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    
    let mut rt = PlatoRuntime::with_config(RuntimeConfig::default())
        .expect("Failed to initialize runtime");
    
    println!("Running optimization cycle...");
    println!();
    
    let result = rt.optimize();
    
    println!("  Optimal thread count: {}", result.thread_count);
    println!("  Best precision: {}", result.best_precision);
    println!("  Peak throughput: {:.2} Mops/s", result.throughput_improvement / 1e6);
    println!();
    
    // Run sizing benchmark
    println!("  Batch size optimization:");
    let sizes = task::benchmark::run_sizing_benchmark();
    task::benchmark::print_sizing_results(&sizes);
}

fn cmd_benchmark() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║       PLATO RUNTIME — Constraint Benchmarks      ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    
    // Warmup
    print!("  Warming up... ");
    let _ = task::constraint::bench_constraint_i32(100_000);
    println!("done");
    println!();
    
    // I32 constraint check
    print!("  INT32 constraint check (1M)... ");
    let (violations, dur) = task::constraint::bench_constraint_i32(1_000_000);
    println!("{:.2} ms ({} violations)", dur.as_secs_f64() * 1000.0, violations);
    
    // Eisenstein
    print!("  Eisenstein filter (100K pairs)... ");
    let pairs: Vec<(i64, i64)> = (0i64..100_000).map(|i| (i, i.wrapping_mul(3))).collect();
    let start = std::time::Instant::now();
    let results = task::constraint::eisenstein_filter(&pairs, 1_000_000_000);
    let dur = start.elapsed();
    println!("{:.2} ms ({} passed)", dur.as_secs_f64() * 1000.0, results.len());
    
    println!();
    
    // Sizing sweep
    println!("  Batch size sweep:");
    let sizes = task::benchmark::run_sizing_benchmark();
    task::benchmark::print_sizing_results(&sizes);
    
    // Find best
    if let Some(best) = sizes.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
        println!();
        println!("  ✓ Optimal batch size: {} ({:.2} Mops/s)", best.0, best.1);
    }
}

fn cmd_idle_harvest() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║       PLATO RUNTIME — Idle Harvester Daemon      ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("Monitoring user activity... (Ctrl+C to stop)");
    println!("Threshold: 30s idle before ramping up");
    println!();
    
    let mut rt = PlatoRuntime::with_config(RuntimeConfig {
        idle_threshold_ms: 30_000,
        idle_budget: 0.8,
        active_budget: 0.2,
        ..Default::default()
    }).expect("Failed to initialize runtime");
    
    loop {
        let budget = rt.tick();
        let idle = rt.is_idle();
        let dur = rt.idle_harvester.idle_duration();
        
        let state_icon = if idle { "⚪" } else { "🔵" };
        let state_name = if idle { "IDLE" } else { "ACTIVE" };
        
        print!("\r  {} {} | idle: {:.0}s | budget: {:.0}% | load: {:.2}   ",
            state_icon, state_name, dur.as_secs_f64(), budget * 100.0,
            rt.idle_harvester.monitor.load_average());
        use std::io::Write;
        std::io::stdout().flush().ok();
        
        std::thread::sleep(Duration::from_secs(1));
    }
}

// Re-exports for command use
use plato_runtime::{discovery, profile, schedule, gpu, task};
