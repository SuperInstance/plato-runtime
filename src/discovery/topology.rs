/// Cache and topology mapping
use std::fs;

pub struct CacheInfo {
    pub l1_data: usize,      // bytes per core
    pub l1_instruction: usize,
    pub l2: usize,           // bytes per core
    pub l3: usize,           // bytes total (shared)
    pub cache_line: usize,
}

impl std::fmt::Display for CacheInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  Cache Hierarchy:")?;
        writeln!(f, "    L1 Data:         {} KB/core", self.l1_data / 1024)?;
        writeln!(f, "    L1 Instruction:  {} KB/core", self.l1_instruction / 1024)?;
        writeln!(f, "    L2:              {} KB/core", self.l2 / 1024)?;
        writeln!(f, "    L3:              {} MB shared", self.l3 / (1024 * 1024))?;
        writeln!(f, "    Cache Line:      {} bytes", self.cache_line)
    }
}

/// Detect cache topology from sysfs
pub fn detect_cache_topology() -> CacheInfo {
    let mut info = CacheInfo {
        l1_data: 32 * 1024,       // Default: 32KB
        l1_instruction: 32 * 1024,
        l2: 512 * 1024,           // Default: 512KB
        l3: 8 * 1024 * 1024,      // Default: 8MB
        cache_line: 64,
    };

    // Try reading from sysfs CPU cache info
    // /sys/devices/system/cpu/cpu0/cache/index{0,1,2,3}/
    for idx in 0..4 {
        let path = format!("/sys/devices/system/cpu/cpu0/cache/index{}/level", idx);
        if let Ok(level_str) = fs::read_to_string(&path) {
            let level: u32 = level_str.trim().parse().unwrap_or(0);
            let size_path = format!("/sys/devices/system/cpu/cpu0/cache/index{}/size", idx);
            let size_str = fs::read_to_string(&size_path).unwrap_or_default();
            let size_bytes = parse_cache_size(&size_str);
            
            let type_path = format!("/sys/devices/system/cpu/cpu0/cache/index{}/type", idx);
            let type_str = fs::read_to_string(&type_path).unwrap_or_default();
            
            match level {
                1 => {
                    if type_str.trim() == "Data" {
                        info.l1_data = size_bytes;
                    } else if type_str.trim() == "Instruction" {
                        info.l1_instruction = size_bytes;
                    }
                }
                2 => info.l2 = size_bytes,
                3 => info.l3 = size_bytes,
                _ => {}
            }
        }
        
        // Cache line size
        let cl_path = format!("/sys/devices/system/cpu/cpu0/cache/index{}/coherency_line_size", idx);
        if let Ok(cl_str) = fs::read_to_string(&cl_path) {
            if let Ok(cl) = cl_str.trim().parse::<usize>() {
                info.cache_line = cl * 1024; // Wait, this is already in bytes
                // Actually sysfs reports in bytes
                info.cache_line = cl;
            }
        }
    }

    info
}

fn parse_cache_size(s: &str) -> usize {
    let s = s.trim();
    if let Some(k) = s.strip_suffix('K') {
        k.parse::<usize>().unwrap_or(0) * 1024
    } else if let Some(m) = s.strip_suffix('M') {
        m.parse::<usize>().unwrap_or(0) * 1024 * 1024
    } else {
        s.parse::<usize>().unwrap_or(0)
    }
}
