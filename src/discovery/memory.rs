/// Memory topology detection
use crate::types::Capabilities;

pub fn discover_memory(caps: &mut Capabilities) {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") && caps.memory_total == 0 {
                if let Some(v) = line.split_whitespace().nth(1) {
                    caps.memory_total = v.parse::<usize>().unwrap_or(0) * 1024;
                }
            }
            if line.starts_with("MemAvailable:") && caps.memory_available == 0 {
                if let Some(v) = line.split_whitespace().nth(1) {
                    caps.memory_available = v.parse::<usize>().unwrap_or(0) * 1024;
                }
            }
            // Detect Huge Pages support
            if line.starts_with("Hugepagesize:") {
                if let Some(v) = line.split_whitespace().nth(1) {
                    let hp_size: usize = v.parse().unwrap_or(0) * 1024;
                    if hp_size > 0 {
                        // Could use huge pages for large constraint arrays
                    }
                }
            }
        }
    }
}

pub fn print_memory_info(caps: &Capabilities) {
    if caps.memory_total > 0 {
        let total_gb = caps.memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
        let avail_gb = caps.memory_available as f64 / (1024.0 * 1024.0 * 1024.0);
        let used_pct = ((caps.memory_total - caps.memory_available) as f64 / caps.memory_total as f64) * 100.0;
        println!("  RAM: {:.1} GB total / {:.1} GB available ({:.0}% used)",
            total_gb, avail_gb, used_pct);
    }
}
