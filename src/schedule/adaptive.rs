/// Adaptive priority scheduler
/// Priority queues: Critical > High > Normal > Low > Idle
/// Affinity-aware task routing

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::types::{Priority, Task, TaskHandle};
use std::time::Instant;

static TASK_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub struct AdaptiveScheduler {
    queues: [Mutex<VecDeque<Task>>; 5], // One per priority level
}

impl AdaptiveScheduler {
    pub fn new() -> Self {
        Self {
            queues: [
                Mutex::new(VecDeque::new()), // Idle
                Mutex::new(VecDeque::new()), // Low
                Mutex::new(VecDeque::new()), // Normal
                Mutex::new(VecDeque::new()), // High
                Mutex::new(VecDeque::new()), // Critical
            ],
        }
    }

    pub fn submit(&self, task: Task, priority: Priority) -> TaskHandle {
        let id = TASK_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let handle = TaskHandle {
            id,
            submitted_at: Instant::now(),
        };
        let idx = priority as usize;
        self.queues[idx].lock().unwrap().push_back(task);
        handle
    }

    pub fn poll(&self) -> Option<(Task, Priority)> {
        // Check from highest to lowest priority
        for level in (0..5).rev() {
            if let Ok(mut q) = self.queues[level].lock() {
                if let Some(task) = q.pop_front() {
                    return Some((task, Priority::try_from(level as u8).unwrap_or(Priority::Normal)));
                }
            }
        }
        None
    }

    pub fn queue_depth(&self) -> usize {
        self.queues.iter().map(|q| q.lock().unwrap().len()).sum()
    }

    pub fn queue_depth_by_priority(&self) -> [usize; 5] {
        [
            self.queues[0].lock().unwrap().len(),
            self.queues[1].lock().unwrap().len(),
            self.queues[2].lock().unwrap().len(),
            self.queues[3].lock().unwrap().len(),
            self.queues[4].lock().unwrap().len(),
        ]
    }
}

impl Priority {
    pub fn try_from(v: u8) -> Result<Self, ()> {
        match v {
            0 => Ok(Self::Idle),
            1 => Ok(Self::Low),
            2 => Ok(Self::Normal),
            3 => Ok(Self::High),
            4 => Ok(Self::Critical),
            _ => Err(()),
        }
    }
}
