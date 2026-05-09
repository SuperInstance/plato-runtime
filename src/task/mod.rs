/// Task definitions
pub mod constraint;
pub mod benchmark;

use std::time::Instant;

#[derive(Debug)]
pub struct TaskResult {
    pub task_id: u64,
    pub completed_at: Instant,
    pub duration_ns: u64,
    pub ops_completed: u64,
}
