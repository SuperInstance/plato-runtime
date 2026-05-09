pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod topology;

use crate::types::ResourceRegistry;
use topology::detect_cache_topology;

/// Run full discovery
pub fn discover_all(gpu_enabled: bool) -> ResourceRegistry {
    let mut registry = ResourceRegistry::new();

    // CPU discovery
    let mut cpu_res = cpu::discover_cpu();
    
    // Enhance with cache topology
    let cache_info = detect_cache_topology();
    cpu_res.capabilities.cache_l1 = Some(cache_info.l1_data);
    // L2 already from cpuinfo or topology
    if cpu_res.capabilities.cache_l2.is_none() {
        cpu_res.capabilities.cache_l2 = Some(cache_info.l2);
    }
    cpu_res.capabilities.cache_l3 = Some(cache_info.l3);
    
    // Memory
    memory::discover_memory(&mut cpu_res.capabilities);
    
    registry.add(cpu_res);

    // GPU discovery
    if gpu_enabled {
        if let Some(gpu_res) = gpu::discover_gpu() {
            registry.add(gpu_res);
        }
    }

    registry
}

pub fn print_discovery(registry: &ResourceRegistry) {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║         PLATO RUNTIME — Resource Discovery       ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    for res in &registry.resources {
        match res.kind {
            crate::types::ResourceKind::CpuCluster | crate::types::ResourceKind::CpuCore => {
                cpu::print_cpu_details(res);
                println!();
                let cache = detect_cache_topology();
                print!("{}", cache);
                println!();
                memory::print_memory_info(&res.capabilities);
            }
            crate::types::ResourceKind::GpuDevice | crate::types::ResourceKind::GpuStream => {
                println!();
                gpu::print_gpu_details(res);
            }
        }
        println!();
    }

    println!("──────────────────────────────────────────────────");
    println!("Total resources discovered: {}", registry.resources.len());
    println!("Available: {} / {}", registry.available().len(), registry.resources.len());
}
