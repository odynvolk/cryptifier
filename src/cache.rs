use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

const SHORT_TTL: usize = 600;
const LONG_TTL: usize = 43200;

pub struct Cache<T: Clone + Send + Sync + 'static> {
    cache: Arc<RwLock<LruCache<String, T>>>,
}

impl<T: Clone + Send + Sync + 'static> Cache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.cache.write().unwrap();
        cache.get_mut(key).map(|v| v.clone())
    }

    pub fn set(&self, key: &str, value: T) {
        let mut cache = self.cache.write().unwrap();
        cache.push(key.to_string(), value);
    }
}

lazy_static::lazy_static! {
    pub static ref SHORT_CACHE: Cache<String> = Cache::new(SHORT_TTL);
    pub static ref LONG_CACHE: Cache<String> = Cache::new(LONG_TTL);
}
