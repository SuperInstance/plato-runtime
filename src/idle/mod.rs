pub mod activity;
pub mod budget;

pub use activity::{ActivityMonitor, BudgetAllocator};
pub use budget::ResourceBudget;

use crate::types::IdleTask;

/// The idle harvester — runs low-priority work when user is inactive
pub struct IdleHarvester {
    pub monitor: ActivityMonitor,
    pub budget: BudgetAllocator,
    pub tasks: Vec<IdleTask>,
}

impl IdleHarvester {
    pub fn new(idle_threshold_ms: u64, idle_budget: f32, active_budget: f32) -> Self {
        use std::time::Duration;
        Self {
            monitor: ActivityMonitor::new(Duration::from_millis(idle_threshold_ms)),
            budget: BudgetAllocator::new(idle_budget, active_budget),
            tasks: vec![
                IdleTask::SelfProfile,
                IdleTask::KernelPrecompile,
                IdleTask::OptimizationSweep,
            ],
        }
    }

    pub fn is_idle(&self) -> bool {
        self.monitor.is_idle()
    }

    pub fn idle_duration(&self) -> std::time::Duration {
        self.monitor.idle_duration()
    }

    pub fn tick(&mut self) -> f32 {
        self.monitor.update_load();
        let idle = self.monitor.is_idle();
        self.budget.update(idle)
    }

    pub fn print_status(&self) {
        let idle = self.monitor.is_idle();
        let dur = self.monitor.idle_duration();
        let load = self.monitor.load_average();
        
        println!("  State: {}", if idle { "⚪ IDLE" } else { "🔵 ACTIVE" });
        println!("  Idle for: {:.1}s", dur.as_secs_f64());
        println!("  Load average: {:.2}", load);
        println!("  Resource budget: {:.0}%", self.budget.current() * 100.0);
        println!("  Idle tasks queued: {}", self.tasks.len());
    }
}
