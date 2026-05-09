/// Budget types for idle harvester
use std::time::Duration;

/// How much of each resource to use during idle time
#[derive(Debug, Clone)]
pub struct ResourceBudget {
    /// Fraction of CPU cores to use (0.0-1.0)
    pub cpu_fraction: f32,
    /// Fraction of GPU to use (0.0-1.0)
    pub gpu_fraction: f32,
    /// Max memory to allocate (bytes)
    pub memory_limit: usize,
    /// Max single task duration
    pub task_timeout: Duration,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            cpu_fraction: 0.8,
            gpu_fraction: 0.8,
            memory_limit: 2 * 1024 * 1024 * 1024, // 2GB
            task_timeout: Duration::from_secs(30),
        }
    }
}

impl ResourceBudget {
    pub fn idle_budget() -> Self {
        Self {
            cpu_fraction: 0.8,
            gpu_fraction: 0.8,
            memory_limit: 2 * 1024 * 1024 * 1024,
            task_timeout: Duration::from_secs(30),
        }
    }

    pub fn active_budget() -> Self {
        Self {
            cpu_fraction: 0.2,
            gpu_fraction: 0.1,
            memory_limit: 512 * 1024 * 1024,
            task_timeout: Duration::from_secs(5),
        }
    }
}
