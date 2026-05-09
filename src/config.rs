/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum CPU threads to use (0 = auto-detect optimal)
    pub max_threads: usize,
    /// Enable GPU
    pub gpu_enabled: bool,
    /// Idle threshold before ramping up background work (ms)
    pub idle_threshold_ms: u64,
    /// Max resource usage when idle (0.0-1.0)
    pub idle_budget: f32,
    /// Max resource usage when user active (0.0-1.0)
    pub active_budget: f32,
    /// Auto-profile on startup
    pub auto_profile: bool,
    /// PLATO server URL
    pub plato_url: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_threads: 0,
            gpu_enabled: true,
            idle_threshold_ms: 30_000,
            idle_budget: 0.8,
            active_budget: 0.2,
            auto_profile: true,
            plato_url: None,
        }
    }
}
