use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::debug_cache;

pub struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

pub struct ResponseCache<T> {
    entry: Arc<Mutex<Option<CacheEntry<T>>>>,
    ttl: Duration,
}

impl<T: Clone> ResponseCache<T> {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            entry: Arc::new(Mutex::new(None)),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self) -> Option<T> {
        let guard = self.entry.lock().ok()?;
        guard.as_ref().and_then(|entry| {
            if Instant::now() < entry.expires_at {
                debug_cache!("Hit: Returning cached data");
                Some(entry.data.clone())
            } else {
                debug_cache!("Miss: Cache entry expired");
                None
            }
        })
    }

    pub fn set(&self, data: T) {
        if let Ok(mut guard) = self.entry.lock() {
            *guard = Some(CacheEntry {
                data,
                expires_at: Instant::now() + self.ttl,
            });
            debug_cache!("Set: Cached data (TTL: {}s)", self.ttl.as_secs());
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.entry.lock() {
            *guard = None;
            debug_cache!("Clear: Cache invalidated");
        }
    }
}
