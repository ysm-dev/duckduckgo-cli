use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use time::OffsetDateTime;

pub type SharedClock = Arc<dyn Clock>;

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> OffsetDateTime;

    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(tokio::time::sleep(duration))
    }
}

#[derive(Debug)]
pub struct ManualClock {
    now: Mutex<OffsetDateTime>,
}

impl ManualClock {
    pub fn new(now: OffsetDateTime) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, duration: Duration) {
        let delta = time::Duration::try_from(duration).unwrap_or(time::Duration::MAX);
        let mut now = self.now.lock().expect("manual clock mutex poisoned");
        *now += delta;
    }

    pub fn now(&self) -> OffsetDateTime {
        *self.now.lock().expect("manual clock mutex poisoned")
    }
}

impl Clock for ManualClock {
    fn now(&self) -> OffsetDateTime {
        Self::now(self)
    }

    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.advance(duration);
            tokio::task::yield_now().await;
        })
    }
}
