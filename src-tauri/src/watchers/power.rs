use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct PowerWatcher {
    is_locked_or_asleep: Arc<AtomicBool>,
}

impl PowerWatcher {
    pub fn new() -> Self {
        Self {
            is_locked_or_asleep: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_locked_or_asleep.load(Ordering::Relaxed)
    }

    pub fn set_paused(&self, paused: bool) {
        self.is_locked_or_asleep.store(paused, Ordering::Relaxed);
    }
}
