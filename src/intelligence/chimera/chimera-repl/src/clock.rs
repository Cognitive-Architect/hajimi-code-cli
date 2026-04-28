//! Clock trait for time abstraction and test injection.
use std::time::SystemTime;

/// Abstract clock for time operations.
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}

/// System clock using std::time::SystemTime.
#[derive(Debug, Clone, Default)]
pub struct SystemTimeClock;

impl Clock for SystemTimeClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    }
}

/// Test clock for testing with fixed timestamps.
#[derive(Debug, Clone, Default)]
pub struct TestClock {
    time_ms: u64,
}

impl TestClock {
    pub fn with_time(time_ms: u64) -> Self {
        Self { time_ms }
    }
    pub fn advance(&mut self, ms: u64) {
        self.time_ms += ms;
    }
}

impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.time_ms
    }
}
