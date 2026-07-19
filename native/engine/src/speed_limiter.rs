use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Notify;
use tokio::time::Instant;

#[derive(Clone)]
pub struct SpeedLimiter {
    inner: Arc<Inner>,
}

struct Inner {
    limit_bps: AtomicU64,
    tokens: AtomicU64,
    notify: Notify,
    refill_wake: Arc<Notify>,
}

const REFILL_INTERVAL_MS: u64 = 50;
const CAP_DURATION_MS: u64 = 250;

impl SpeedLimiter {
    pub fn new(limit_bps: u64) -> Self {
        Self {
            inner: Arc::new(Inner {
                limit_bps: AtomicU64::new(limit_bps),
                tokens: AtomicU64::new(0),
                notify: Notify::new(),
                refill_wake: Arc::new(Notify::new()),
            }),
        }
    }

    pub fn set_limit(&self, limit_bps: u64) {
        self.inner.limit_bps.store(limit_bps, Ordering::Relaxed);
        self.inner.notify.notify_waiters();
        self.inner.refill_wake.notify_one();
    }

    pub fn limit(&self) -> u64 {
        self.inner.limit_bps.load(Ordering::Relaxed)
    }

    pub async fn consume(&self, requested: u64) -> u64 {
        if requested == 0 {
            return 0;
        }
        loop {
            let limit = self.inner.limit_bps.load(Ordering::Relaxed);
            if limit == 0 {
                return requested;
            }
            let available = self.inner.tokens.load(Ordering::Acquire);
            if available > 0 {
                let take = requested.min(available);
                match self.inner.tokens.compare_exchange_weak(
                    available,
                    available - take,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return take,
                    Err(_) => continue,
                }
            }
            tokio::select! {
                biased;
                () = self.inner.notify.notified() => {}
                () = tokio::time::sleep(std::time::Duration::from_millis(REFILL_INTERVAL_MS + 10)) => {}
            }
        }
    }

    pub fn spawn_refill_task(&self) {
        let weak = Arc::downgrade(&self.inner);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(REFILL_INTERVAL_MS));
            interval.tick().await;
            let mut last_refill = Instant::now();
            loop {
                interval.tick().await;
                let Some(inner) = weak.upgrade() else { break };
                let limit = inner.limit_bps.load(Ordering::Relaxed);
                if limit == 0 {
                    inner.tokens.store(0, Ordering::Relaxed);
                    inner.notify.notify_waiters();
                    let refill_wake = Arc::clone(&inner.refill_wake);
                    drop(inner);
                    let wake = refill_wake.notified();
                    tokio::select! {
                        () = tokio::time::sleep(std::time::Duration::from_secs(2)) => {}
                        () = wake => {}
                    }
                    last_refill = Instant::now();
                    continue;
                }
                let now = Instant::now();
                let elapsed_us = (now - last_refill).as_micros() as u64;
                let refill = ((limit as u128) * (elapsed_us as u128) / 1_000_000u128) as u64;
                if refill == 0 { continue; }
                last_refill = now;
                let nominal_tick = limit * REFILL_INTERVAL_MS / 1000;
                let cap = (limit * CAP_DURATION_MS / 1000).max(nominal_tick * 2).max(1);
                loop {
                    let current = inner.tokens.load(Ordering::Acquire);
                    let new_val = current.saturating_add(refill).min(cap);
                    if new_val == current { break; }
                    match inner.tokens.compare_exchange_weak(current, new_val, Ordering::AcqRel, Ordering::Relaxed) {
                        Ok(_) => break,
                        Err(_) => continue,
                    }
                }
                inner.notify.notify_waiters();
            }
        });
    }
}
