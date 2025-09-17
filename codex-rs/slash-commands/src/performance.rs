use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static REGISTRY_LOAD_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn record_load_metrics() {
    REGISTRY_LOAD_COUNT.fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
pub fn load_count() -> u64 {
    REGISTRY_LOAD_COUNT.load(Ordering::Relaxed)
}
