/// CUDA backend — dynamic loading of CUDA driver API
/// Uses libcuda.so.1 which is available in WSL2

use std::path::Path;

/// Check if CUDA driver is available
pub fn is_cuda_available() -> bool {
    let driver_paths = [
        "/usr/lib/wsl/lib/libcuda.so.1",
        "/usr/lib/wsl/lib/libcuda.so",
        "/usr/lib/x86_64-linux-gnu/libcuda.so.1",
        "/usr/local/cuda/lib64/libcuda.so.1",
    ];
    let has_driver = driver_paths.iter().any(|p| Path::new(p).exists());
    // Also check device nodes (more reliable in WSL2)
    let has_device = Path::new("/dev/nvidia0").exists()
        || Path::new("/proc/driver/nvidia").exists();
    has_driver || has_device
}

/// CUDA device info (gathered without full CUDA runtime)
#[derive(Debug)]
pub struct CudaDeviceInfo {
    pub compute_capability: (u32, u32),
    pub sm_count: u32,
    pub memory_bytes: usize,
    pub name: String,
}

impl Default for CudaDeviceInfo {
    fn default() -> Self {
        // Known defaults for RTX 4050 Laptop
        Self {
            compute_capability: (8, 6),
            sm_count: 16,
            memory_bytes: 6 * 1024 * 1024 * 1024,
            name: "NVIDIA GeForce RTX 4050 Laptop GPU".into(),
        }
    }
}

/// Get device info (using known defaults; nvcc gives toolkit version, not compute cap)
pub fn get_device_info() -> Option<CudaDeviceInfo> {
    if !is_cuda_available() {
        return None;
    }
    
    let mut info = CudaDeviceInfo::default();
    
    // Try nvidia-smi for real compute capability (available on most systems)
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=compute_cap", "--format=csv,noheader"])
        .output()
    {
        let cap = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<u32> = cap.trim().split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        if parts.len() >= 2 {
            info.compute_capability = (parts[0], parts[1]);
        }
    }
    // NOTE: Do NOT parse nvcc --version — it reports CUDA toolkit version, not GPU capability

    Some(info)
}

pub fn print_cuda_info(info: &CudaDeviceInfo) {
    println!("  Device: {}", info.name);
    println!("  Compute: sm_{}{}", info.compute_capability.0, info.compute_capability.1);
    println!("  SMs: {}", info.sm_count);
    println!("  VRAM: {:.1} GB", info.memory_bytes as f64 / 1e9);
}
