use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPhase {
    Working,
    PreBreak,
    Break,
    Paused,
    Snoozed,
}

#[derive(Debug, Clone)]
pub enum TimerState {
    Working {
        started_at: Instant,
        duration: Duration,
    },
    PreBreak {
        started_at: Instant,
        duration: Duration,
    },
    Break {
        started_at: Instant,
        duration: Duration,
    },
    Paused {
        remaining_work: Duration,
    },
    Snoozed {
        until: Instant,
    },
}

pub struct StateMachine {
    pub current: TimerState,
    pub work_duration: Duration,
    pub break_duration: Duration,
    pub prebreak_duration: Duration,
}

impl StateMachine {
    pub fn new(work_interval_mins: u32, break_duration_secs: u32) -> Self {
        let work_duration = Duration::from_secs((work_interval_mins as u64) * 60);
        let break_duration = Duration::from_secs(break_duration_secs as u64);
        let prebreak_duration = Duration::from_secs(10);

        Self {
            current: TimerState::Working {
                started_at: Instant::now(),
                duration: work_duration,
            },
            work_duration,
            break_duration,
            prebreak_duration,
        }
    }

    pub fn update_durations(&mut self, work_mins: u32, break_secs: u32) {
        self.work_duration = Duration::from_secs((work_mins as u64) * 60);
        self.break_duration = Duration::from_secs(break_secs as u64);
    }

    pub fn phase(&self) -> TimerPhase {
        match self.current {
            TimerState::Working { .. } => TimerPhase::Working,
            TimerState::PreBreak { .. } => TimerPhase::PreBreak,
            TimerState::Break { .. } => TimerPhase::Break,
            TimerState::Paused { .. } => TimerPhase::Paused,
            TimerState::Snoozed { .. } => TimerPhase::Snoozed,
        }
    }

    /// Reset working timer (e.g. idle detected for 2+ min)
    pub fn reset_working(&mut self) {
        self.current = TimerState::Working {
            started_at: Instant::now(),
            duration: self.work_duration,
        };
    }

    /// Trigger break immediately
    pub fn trigger_break_now(&mut self) {
        self.current = TimerState::Break {
            started_at: Instant::now(),
            duration: self.break_duration,
        };
    }

    /// Skip break -> returns to Working
    pub fn skip_break(&mut self) {
        self.reset_working();
    }

    /// Pause the timer
    pub fn pause(&mut self) {
        if let TimerState::Working { started_at, duration } = self.current {
            let elapsed = started_at.elapsed();
            let remaining = if duration > elapsed {
                duration - elapsed
            } else {
                Duration::from_secs(0)
            };
            self.current = TimerState::Paused {
                remaining_work: remaining,
            };
        }
    }

    /// Resume the timer from paused state
    pub fn resume(&mut self) {
        if let TimerState::Paused { remaining_work } = self.current {
            self.current = TimerState::Working {
                started_at: Instant::now(),
                duration: remaining_work,
            };
        } else if let TimerState::Snoozed { .. } = self.current {
            self.reset_working();
        }
    }

    /// Snooze for a specific duration
    pub fn snooze(&mut self, duration: Duration) {
        self.current = TimerState::Snoozed {
            until: Instant::now() + duration,
        };
    }

    /// Calculate remaining seconds in current state
    pub fn remaining_seconds(&self) -> u64 {
        match self.current {
            TimerState::Working { started_at, duration } => {
                let elapsed = started_at.elapsed();
                duration.saturating_sub(elapsed).as_secs()
            }
            TimerState::PreBreak { started_at, duration } => {
                let elapsed = started_at.elapsed();
                duration.saturating_sub(elapsed).as_secs()
            }
            TimerState::Break { started_at, duration } => {
                let elapsed = started_at.elapsed();
                duration.saturating_sub(elapsed).as_secs()
            }
            TimerState::Paused { remaining_work } => remaining_work.as_secs(),
            TimerState::Snoozed { until } => {
                let now = Instant::now();
                if until > now {
                    (until - now).as_secs()
                } else {
                    0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_working() {
        let sm = StateMachine::new(20, 20);
        assert_eq!(sm.phase(), TimerPhase::Working);
        assert!(sm.remaining_seconds() <= 1200);
        assert!(sm.remaining_seconds() >= 1198);
    }

    #[test]
    fn test_trigger_break_now() {
        let mut sm = StateMachine::new(20, 20);
        sm.trigger_break_now();
        assert_eq!(sm.phase(), TimerPhase::Break);
        assert!(sm.remaining_seconds() <= 20);
    }

    #[test]
    fn test_skip_break_returns_to_working() {
        let mut sm = StateMachine::new(20, 20);
        sm.trigger_break_now();
        assert_eq!(sm.phase(), TimerPhase::Break);
        sm.skip_break();
        assert_eq!(sm.phase(), TimerPhase::Working);
    }

    #[test]
    fn test_pause_and_resume() {
        let mut sm = StateMachine::new(20, 20);
        sm.pause();
        assert_eq!(sm.phase(), TimerPhase::Paused);
        let rem_paused = sm.remaining_seconds();
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(sm.remaining_seconds(), rem_paused); // Zero drift while paused!
        sm.resume();
        assert_eq!(sm.phase(), TimerPhase::Working);
    }

    #[test]
    fn test_snooze() {
        let mut sm = StateMachine::new(20, 20);
        sm.snooze(Duration::from_secs(300));
        assert_eq!(sm.phase(), TimerPhase::Snoozed);
        assert!(sm.remaining_seconds() <= 300);
        assert!(sm.remaining_seconds() >= 298);
    }

    #[test]
    fn test_idle_reset() {
        let mut sm = StateMachine::new(20, 20);
        sm.reset_working();
        assert_eq!(sm.phase(), TimerPhase::Working);
        assert!(sm.remaining_seconds() <= 1200);
    }
}

