use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: u64,
}

#[derive(Clone)]
pub struct Cache<T: Clone + Send + Sync + 'static> {
    cache: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl_seconds: u64,
}

impl<T: Clone + Send + Sync + 'static> Cache<T> {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl_seconds,
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.read().unwrap();
        let now = Self::now();

        // Check if entry exists and hasn't expired
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > now {
                return Some(entry.value.clone());
            }
        }

        // Entry doesn't exist or has expired
        None
    }

    pub fn set(&self, key: &str, value: T) {
        let now = Self::now();
        let expires_at = now + self.ttl_seconds;

        let mut cache = self.cache.write().unwrap();
        cache.insert(key.to_string(), CacheEntry { value, expires_at });
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

/// Short-term cache with 10-minute TTL.
pub static SHORT_CACHE: Lazy<Cache<String>> = Lazy::new(|| Cache::new(600));

/// Long-term cache with 12-hour TTL.
pub static LONG_CACHE: Lazy<Cache<String>> = Lazy::new(|| Cache::new(43200));
