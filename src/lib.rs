//! # PLATO Runtime
//! 
//! A self-discovering, self-optimizing compute runtime.
//! Discovers all hardware resources, profiles them, and dynamically schedules
//! constraint-theory workloads with idle harvesting.

pub mod config;
pub mod discovery;
pub mod profile;
pub mod schedule;
pub mod idle;
pub mod task;
pub mod gpu;
pub mod types;

pub use config::RuntimeConfig;
pub use types::*;
pub use discovery::discover_all;
pub use schedule::AdaptiveScheduler;

use std::time::Instant;

/// The main runtime handle
pub struct PlatoRuntime {
    pub resources: ResourceRegistry,
    pub scheduler: AdaptiveScheduler,
    pub idle_harvester: idle::IdleHarvester,
    pub config: RuntimeConfig,
    started_at: Instant,
}

impl PlatoRuntime {
    /// Create with auto-discovery
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(RuntimeConfig::default())
    }

    /// Create with config
    pub fn with_config(config: RuntimeConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let resources = discover_all(config.gpu_enabled);
        
        let idle_harvester = idle::IdleHarvester::new(
            config.idle_threshold_ms,
            config.idle_budget,
            config.active_budget,
        );

        Ok(Self {
            resources,
            scheduler: AdaptiveScheduler::new(),
            idle_harvester,
            config,
            started_at: Instant::now(),
        })
    }

    /// Get discovered resources
    pub fn resources(&self) -> &ResourceRegistry {
        &self.resources
    }

    /// Check if user is idle
    pub fn is_idle(&self) -> bool {
        self.idle_harvester.is_idle()
    }

    /// Get current resource utilization
    pub fn utilization(&self) -> UtilizationReport {
        let load = self.idle_harvester.monitor.load_average();
        let online = schedule::num_online_cpus();
        
        UtilizationReport {
            cpu_percent: (load / online as f64) * 100.0,
            gpu_percent: 0.0, // Would need CUDA API
            memory_used_bytes: 0,
            memory_total_bytes: self.resources.resources.iter()
                .filter_map(|r| if r.capabilities.memory_total > 0 {
                    Some(r.capabilities.memory_total)
                } else { None })
                .next()
                .unwrap_or(0),
            tasks_running: 0,
            tasks_queued: self.scheduler.queue_depth(),
            idle: self.idle_harvester.is_idle(),
        }
    }

    /// Run self-optimization cycle
    pub fn optimize(&mut self) -> OptimizationResult {
        let profile_result = profile::run_profile();
        
        // Update resource profiles
        for res in &mut self.resources.resources {
            res.profile.constraint_check_i8 = profile_result.constraint_check_i8;
            res.profile.constraint_check_i32 = profile_result.constraint_check_i32;
            res.profile.constraint_check_fp64 = profile_result.constraint_check_fp64;
            res.profile.memory_bandwidth = profile_result.memory_bandwidth;
            res.profile.kernel_launch_latency_ns = profile_result.kernel_launch_latency_ns;
            res.profile.tsc_frequency_hz = profile_result.tsc_frequency_hz;
            res.profile.profiled_at = Instant::now();
        }

        let thread_count = schedule::optimal_thread_count();
        let best_throughput = profile_result.constraint_check_i32.max(profile_result.constraint_check_fp64);

        OptimizationResult {
            kernels_optimized: 0,
            batch_sizes_tuned: 0,
            thread_count,
            best_precision: if profile_result.constraint_check_i32 > profile_result.constraint_check_fp64 {
                "INT32".into()
            } else {
                "FP64".into()
            },
            throughput_improvement: best_throughput,
        }
    }

    /// Run the idle harvester tick
    pub fn tick(&mut self) -> f32 {
        self.idle_harvester.tick()
    }

    /// Uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    /// Graceful shutdown
    pub fn shutdown(self) {
        // Drain queues
        let depth = self.scheduler.queue_depth();
        if depth > 0 {
            eprintln!("Warning: {} tasks still in queue on shutdown", depth);
        }
    }
}
