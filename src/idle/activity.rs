/// Idle detection — monitor user activity to determine when to ramp up background work

use std::time::{Duration, Instant};

/// User activity monitor
pub struct ActivityMonitor {
    last_activity: Instant,
    idle_threshold: Duration,
    load_avg: f64,
}

impl ActivityMonitor {
    pub fn new(idle_threshold: Duration) -> Self {
        Self {
            last_activity: Instant::now(),
            idle_threshold,
            load_avg: 0.0,
        }
    }

    /// Record user activity
    pub fn record_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Update system load average from /proc/loadavg
    pub fn update_load(&mut self) {
        if let Ok(s) = std::fs::read_to_string("/proc/loadavg") {
            self.load_avg = s.split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(self.load_avg);
        }
    }

    /// Check if user is idle
    pub fn is_idle(&self) -> bool {
        self.last_activity.elapsed() > self.idle_threshold
    }

    /// Time since last activity
    pub fn idle_duration(&self) -> Duration {
        self.last_activity.elapsed()
    }

    /// Get current load average
    pub fn load_average(&self) -> f64 {
        self.load_avg
    }
}

/// Resource budget allocator
pub struct BudgetAllocator {
    pub idle_budget: f32,
    pub active_budget: f32,
    is_idle: bool,
    current_budget: f32,
    ramp_direction: f32, // positive = ramping up, negative = ramping down
}

impl BudgetAllocator {
    pub fn new(idle_budget: f32, active_budget: f32) -> Self {
        Self {
            idle_budget,
            active_budget,
            is_idle: false,
            current_budget: active_budget,
            ramp_direction: 0.0,
        }
    }

    /// Update budget based on idle state, with smooth transitions
    pub fn update(&mut self, idle: bool) -> f32 {
        if idle != self.is_idle {
            self.is_idle = idle;
            self.ramp_direction = if idle { 0.05 } else { -0.05 };
        }

        let target = if idle { self.idle_budget } else { self.active_budget };
        
        // Smooth ramp over ~1s at 20 updates/sec
        if (self.current_budget - target).abs() > 0.01 {
            self.current_budget += self.ramp_direction;
            self.current_budget = self.current_budget.clamp(
                self.active_budget.min(self.idle_budget),
                self.active_budget.max(self.idle_budget),
            );
        } else {
            self.current_budget = target;
        }

        self.current_budget
    }

    pub fn current(&self) -> f32 {
        self.current_budget
    }
}
