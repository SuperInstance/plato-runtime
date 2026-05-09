/// CPU feature detection via /proc/cpuinfo
use crate::types::{Capabilities, ComputeResource, PerformanceProfile, ResourceId, ResourceKind, ResourceStatus};

pub fn discover_cpu() -> ComputeResource {
    let mut caps = Capabilities::default();
    
    // Read /proc/cpuinfo
    if let Ok(info) = std::fs::read_to_string("/proc/cpuinfo") {
        let mut core_count = 0u32;
        let mut processor_count = 0u32;
        let mut flags = String::new();
        let mut _model_name = String::new();
        let mut cache_size = String::new();

        for line in info.lines() {
            if let Some(val) = line.strip_prefix("processor") {
                if let Some(v) = val.trim_start_matches(|c: char| !c.is_ascii_digit())
                    .split_whitespace().next()
                {
                    processor_count = v.parse().unwrap_or(0) + 1;
                }
            }
            if line.starts_with("cpu cores") {
                if let Some(v) = line.split(':').nth(1) {
                    core_count = v.trim().parse().unwrap_or(0);
                }
            }
            if line.starts_with("model name") {
                if let Some(v) = line.split(':').nth(1) {
                    _model_name = v.trim().to_string();
                }
            }
            if line.starts_with("flags") {
                if let Some(v) = line.split(':').nth(1) {
                    flags = v.trim().to_string();
                }
            }
            if line.starts_with("cache size") {
                if let Some(v) = line.split(':').nth(1) {
                    cache_size = v.trim().to_string();
                }
            }
        }

        caps.cores = Some(core_count);
        caps.threads = Some(processor_count);
        
        let flag_list: Vec<&str> = flags.split_whitespace().collect();
        let has = |f: &str| flag_list.contains(&f);
        
        caps.avx512 = has("avx512f");
        caps.avx512_vnni = has("avx512_vnni");
        caps.avx512_ifma = has("avx512ifma");
        caps.avx512_bf16 = has("avx512_bf16");
        caps.avx512_vbmi2 = has("avx512_vbmi2");
        caps.avx512_vpopcntdq = has("avx512_vpopcntdq");
        caps.avx512_vp2intersect = has("avx512_vp2intersect");
        caps.avx512_bitalg = has("avx512_bitalg");
        caps.avx2 = has("avx2");
        caps.avx_vnni = has("avx_vnni");

        // Cache sizes from cpuinfo (per-core L2 reported as "cache size")
        if let Some(kb_str) = cache_size.strip_suffix(" KB") {
            if let Ok(kb) = kb_str.parse::<usize>() {
                caps.cache_l2 = Some(kb * 1024);
            }
        }
    }

    // Detect total memory from /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(v) = line.split_whitespace().nth(1) {
                    caps.memory_total = v.parse::<usize>().unwrap_or(0) * 1024;
                }
            }
            if line.starts_with("MemAvailable:") {
                if let Some(v) = line.split_whitespace().nth(1) {
                    caps.memory_available = v.parse::<usize>().unwrap_or(0) * 1024;
                }
            }
        }
    }

    let label = caps.cores
        .map(|c| format!("CPU Cluster ({}C/{}T)", c, caps.threads.unwrap_or(c * 2)))
        .unwrap_or_else(|| "CPU Cluster".into());

    ComputeResource {
        id: ResourceId(label.clone()),
        kind: ResourceKind::CpuCluster,
        capabilities: caps,
        profile: PerformanceProfile::default(),
        status: ResourceStatus::Available,
    }
}

pub fn print_cpu_details(res: &ComputeResource) {
    let c = &res.capabilities;
    println!("  {}", res.id);
    println!("  Status: {}", res.status);
    if let Some(name) = &c.cores {
        println!("  Cores: {} / Threads: {}", name, c.threads.unwrap_or(0));
    }
    println!("  AVX-512:       {}", if c.avx512 { "✓" } else { "✗" });
    if c.avx512 {
        println!("    VNNI:        {}", if c.avx512_vnni { "✓" } else { "✗" });
        println!("    IFMA:        {}", if c.avx512_ifma { "✓" } else { "✗" });
        println!("    BF16:        {}", if c.avx512_bf16 { "✓" } else { "✗" });
        println!("    VBMI2:       {}", if c.avx512_vbmi2 { "✓" } else { "✗" });
        println!("    VPOPCNTDQ:   {}", if c.avx512_vpopcntdq { "✓" } else { "✗" });
        println!("    VP2INTERSECT:{}", if c.avx512_vp2intersect { "✓" } else { "✗" });
        println!("    BITALG:      {}", if c.avx512_bitalg { "✓" } else { "✗" });
    }
    println!("  AVX2:           {}", if c.avx2 { "✓" } else { "✗" });
    println!("  AVX-VNNI:       {}", if c.avx_vnni { "✓" } else { "✗" });
    if let Some(l2) = c.cache_l2 {
        println!("  L2 Cache:       {} KB", l2 / 1024);
    }
    if c.memory_total > 0 {
        println!("  Memory:         {:.1} GB total / {:.1} GB available",
            c.memory_total as f64 / 1e9, c.memory_available as f64 / 1e9);
    }
}
