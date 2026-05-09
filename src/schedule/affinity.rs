/// CPU affinity / core pinning via libc sched_setaffinity

/// Pin current thread to specific CPU core
pub fn pin_to_core(core_id: usize) -> bool {
    #[cfg(target_os = "linux")]
    {
        let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        unsafe {
            libc::CPU_SET(core_id, &mut cpu_set);
        }
        let pid = 0; // current thread
        let result = unsafe {
            libc::sched_setaffinity(pid, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set)
        };
        result == 0
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = core_id;
        false
    }
}

/// Get number of online CPUs
pub fn num_online_cpus() -> usize {
    #[cfg(target_os = "linux")]
    {
        // Try sched_getaffinity first
        let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &mut cpu_set)
        };
        if result == 0 {
            let mut count = 0;
            for i in 0..256 {
                if unsafe { libc::CPU_ISSET(i, &cpu_set) } {
                    count += 1;
                }
            }
            if count > 0 {
                return count;
            }
        }
    }
    num_cpus()
}

/// Fallback: read from /proc/cpuinfo or sysfs
fn num_cpus() -> usize {
    if let Ok(s) = std::fs::read_to_string("/sys/devices/system/cpu/online") {
        // Format: "0-23" or "0,2,4-8"
        let s = s.trim();
        return parse_cpu_list(s);
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn parse_cpu_list(s: &str) -> usize {
    let mut count = 0;
    for part in s.split(',') {
        let part = part.trim();
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(start), Ok(end)) = (a.parse::<usize>(), b.parse::<usize>()) {
                count += end - start + 1;
            }
        } else if let Ok(_) = part.parse::<usize>() {
            count += 1;
        }
    }
    if count == 0 { 4 } else { count }
}

/// Suggest optimal thread count for this machine
pub fn optimal_thread_count() -> usize {
    let cpus = num_online_cpus();
    // For compute-heavy workloads, use physical cores
    // On Zen 5 12C/24T, 12-16 threads is typically optimal
    let physical = cpus / 2; // Assume hyperthreading
    if physical >= 4 { physical.min(cpus) } else { cpus }
}

pub fn print_affinity_info() {
    let online = num_online_cpus();
    let optimal = optimal_thread_count();
    println!("  Online CPUs:     {}", online);
    println!("  Optimal threads: {}", optimal);
    
    // Show current affinity
    #[cfg(target_os = "linux")]
    {
        let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &mut cpu_set)
        };
        if result == 0 {
            let mut cores = Vec::new();
            for i in 0..256 {
                if unsafe { libc::CPU_ISSET(i, &cpu_set) } {
                    cores.push(i);
                }
            }
            println!("  Current affinity: {} cores", cores.len());
        }
    }
}
