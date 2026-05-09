/// Work-stealing deque (simplified)
/// Each worker thread has its own deque; idle workers steal from others

use std::collections::VecDeque;
use std::sync::Mutex;

pub struct WorkStealingDeque<T> {
    local: Mutex<VecDeque<T>>,
}

impl<T> WorkStealingDeque<T> {
    pub fn new() -> Self {
        Self {
            local: Mutex::new(VecDeque::new()),
        }
    }

    /// Push to bottom (owner thread)
    pub fn push(&self, item: T) {
        self.local.lock().unwrap().push_back(item);
    }

    /// Pop from bottom (owner thread, LIFO)
    pub fn pop(&self) -> Option<T> {
        self.local.lock().unwrap().pop_back()
    }

    /// Steal from top (thief thread, FIFO)
    pub fn steal(&self) -> Option<T> {
        self.local.lock().unwrap().pop_front()
    }

    pub fn len(&self) -> usize {
        self.local.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.local.lock().unwrap().is_empty()
    }
}

/// A pool of work-stealing deques
pub struct WorkStealingPool<T> {
    deques: Vec<WorkStealingDeque<T>>,
    next_victim: std::sync::atomic::AtomicUsize,
}

impl<T> WorkStealingPool<T> {
    pub fn new(num_workers: usize) -> Self {
        Self {
            deques: (0..num_workers).map(|_| WorkStealingDeque::new()).collect(),
            next_victim: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn worker_deque(&self, worker_id: usize) -> &WorkStealingDeque<T> {
        &self.deques[worker_id % self.deques.len()]
    }

    pub fn steal_from_any(&self, thief_id: usize) -> Option<T> {
        let n = self.deques.len();
        let start = self.next_victim.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % n;
        for i in 0..n {
            let victim = (start + i) % n;
            if victim != thief_id {
                if let Some(item) = self.deques[victim].steal() {
                    return Some(item);
                }
            }
        }
        None
    }

    pub fn total_len(&self) -> usize {
        self.deques.iter().map(|d| d.len()).sum()
    }
}
