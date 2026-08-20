use cryptifier::cache::Cache;

#[test]
fn cache_returns_stored_value_within_ttl() {
    let mut cache = Cache::<String>::new(3600);
    cache.set("key", "value".to_string());
    assert_eq!(cache.get("key"), Some("value".to_string()));
}

#[test]
fn cache_returns_none_for_missing_key() {
    let cache = Cache::<String>::new(3600);
    assert_eq!(cache.get("missing"), None);
}

#[test]
fn cache_returns_none_for_expired_entry() {
    let mut cache = Cache::<String>::new(0); // TTL 0 -> entry expires immediately
    cache.set("key", "value".to_string());
    assert_eq!(cache.get("key"), None);
}

#[test]
fn cache_overwrites_existing_value() {
    let mut cache = Cache::<String>::new(3600);
    cache.set("key", "first".to_string());
    cache.set("key", "second".to_string());
    assert_eq!(cache.get("key"), Some("second".to_string()));
}
