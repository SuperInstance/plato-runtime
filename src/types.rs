use std::fmt;
use std::time::Instant;

/// Unique resource identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ResourceId(pub String);

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Kind of compute resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    CpuCore,
    CpuCluster,
    GpuDevice,
    GpuStream,
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CpuCore => write!(f, "CPU Core"),
            Self::CpuCluster => write!(f, "CPU Cluster"),
            Self::GpuDevice => write!(f, "GPU Device"),
            Self::GpuStream => write!(f, "GPU Stream"),
        }
    }
}

/// Resource status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceStatus {
    Available,
    Busy,
    Idle,
    Offline,
}

impl fmt::Display for ResourceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "✓ Available"),
            Self::Busy => write!(f, "● Busy"),
            Self::Idle => write!(f, "○ Idle"),
            Self::Offline => write!(f, "✗ Offline"),
        }
    }
}

/// Discovered capabilities
#[derive(Debug, Clone)]
pub struct Capabilities {
    pub cores: Option<u32>,
    pub threads: Option<u32>,
    pub avx512: bool,
    pub avx512_vnni: bool,
    pub avx512_ifma: bool,
    pub avx512_bf16: bool,
    pub avx512_vbmi2: bool,
    pub avx512_vpopcntdq: bool,
    pub avx512_vp2intersect: bool,
    pub avx512_bitalg: bool,
    pub avx2: bool,
    pub avx_vnni: bool,
    pub cache_l1: Option<usize>,
    pub cache_l2: Option<usize>,
    pub cache_l3: Option<usize>,
    pub cuda_compute: Option<(u32, u32)>,
    pub gpu_memory: Option<usize>,
    pub gpu_sm_count: Option<u32>,
    pub memory_total: usize,
    pub memory_available: usize,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            cores: None,
            threads: None,
            avx512: false,
            avx512_vnni: false,
            avx512_ifma: false,
            avx512_bf16: false,
            avx512_vbmi2: false,
            avx512_vpopcntdq: false,
            avx512_vp2intersect: false,
            avx512_bitalg: false,
            avx2: false,
            avx_vnni: false,
            cache_l1: None,
            cache_l2: None,
            cache_l3: None,
            cuda_compute: None,
            gpu_memory: None,
            gpu_sm_count: None,
            memory_total: 0,
            memory_available: 0,
        }
    }
}

/// Performance profile from microbenchmarks
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    pub constraint_check_i8: f64,
    pub constraint_check_i32: f64,
    pub constraint_check_fp64: f64,
    pub eisenstein_norm: f64,
    pub eisenstein_mul: f64,
    pub memory_bandwidth: f64,
    pub kernel_launch_latency_ns: f64,
    pub tsc_frequency_hz: f64,
    pub profiled_at: Instant,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self {
            constraint_check_i8: 0.0,
            constraint_check_i32: 0.0,
            constraint_check_fp64: 0.0,
            eisenstein_norm: 0.0,
            eisenstein_mul: 0.0,
            memory_bandwidth: 0.0,
            kernel_launch_latency_ns: 0.0,
            tsc_frequency_hz: 0.0,
            profiled_at: Instant::now(),
        }
    }
}

/// A discovered compute resource
#[derive(Debug, Clone)]
pub struct ComputeResource {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub capabilities: Capabilities,
    pub profile: PerformanceProfile,
    pub status: ResourceStatus,
}

/// Utilization report
#[derive(Debug, Clone)]
pub struct UtilizationReport {
    pub cpu_percent: f64,
    pub gpu_percent: f64,
    pub memory_used_bytes: usize,
    pub memory_total_bytes: usize,
    pub tasks_running: usize,
    pub tasks_queued: usize,
    pub idle: bool,
}

/// Optimization result
#[derive(Debug)]
pub struct OptimizationResult {
    pub kernels_optimized: usize,
    pub batch_sizes_tuned: usize,
    pub thread_count: usize,
    pub best_precision: String,
    pub throughput_improvement: f64,
}

/// Task handle for submitted tasks
#[derive(Debug)]
pub struct TaskHandle {
    pub id: u64,
    pub submitted_at: Instant,
}

/// Task types
pub enum Task {
    ConstraintCheck {
        data: Vec<u8>,
        precision: Precision,
        priority: Priority,
    },
    Benchmark {
        duration_ms: u64,
    },
    PlatoTile {
        tile_id: String,
    },
    Custom {
        name: String,
        work: Box<dyn Fn() + Send>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Precision {
    I8,
    I32,
    FP64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Idle = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Resource registry holding all discovered resources
#[derive(Debug)]
pub struct ResourceRegistry {
    pub resources: Vec<ComputeResource>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self { resources: Vec::new() }
    }

    pub fn add(&mut self, resource: ComputeResource) {
        self.resources.push(resource);
    }

    pub fn cpu_resources(&self) -> Vec<&ComputeResource> {
        self.resources.iter().filter(|r| matches!(r.kind, ResourceKind::CpuCore | ResourceKind::CpuCluster)).collect()
    }

    pub fn gpu_resources(&self) -> Vec<&ComputeResource> {
        self.resources.iter().filter(|r| matches!(r.kind, ResourceKind::GpuDevice)).collect()
    }

    pub fn available(&self) -> Vec<&ComputeResource> {
        self.resources.iter().filter(|r| r.status == ResourceStatus::Available).collect()
    }
}

/// Idle task types
#[allow(dead_code)]
pub enum IdleTask {
    SelfProfile,
    KernelPrecompile,
    PlatoTileProcessing,
    OptimizationSweep,
    Custom(Box<dyn Fn() + Send>),
}

/// Activity monitor for idle detection
pub struct ActivityMonitor {
    pub last_activity: Instant,
    pub load_average: f64,
}

impl Default for ActivityMonitor {
    fn default() -> Self {
        Self {
            last_activity: Instant::now(),
            load_average: 0.0,
        }
    }
}
