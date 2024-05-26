use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct Store {
    data: Arc<RwLock<HashMap<String, String>>>,
    expirations: Arc<RwLock<HashMap<String, Instant>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: Arc::new(RwLock::new(HashMap::new())),
            expirations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, key: String, value: String, ttl: Option<Duration>) {
        let mut data = self.data.write().unwrap();
        let mut expirations = self.expirations.write().unwrap();
        data.insert(key.clone(), value);
        if let Some(ttl) = ttl {
            expirations.insert(key, Instant::now() + ttl);
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().unwrap();
        let expirations = self.expirations.read().unwrap();
        if let Some(expiration) = expirations.get(key) {
            if Instant::now() >= *expiration {
                return None;
            }
        }
        data.get(key).cloned()
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Store {
            data: self.data.clone(),
            expirations: self.expirations.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_insert_and_get() {
        let store = Store::new();
        store.insert("key1".to_string(), "value1".to_string(), None);

        assert_eq!(store.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_insert_with_ttl() {
        let store = Store::new();
        store.insert(
            "key2".to_string(),
            "value2".to_string(),
            Some(Duration::from_secs(1)),
        );

        assert_eq!(store.get("key2"), Some("value2".to_string()));
    }

    #[test]
    fn test_remove() {
        let store = Store::new();
        store.insert("key1".to_string(), "value1".to_string(), None);

        assert_eq!(store.remove("key1"), Some("value1".to_string()));
        assert_eq!(store.get("key1"), None);
    }

    #[test]
    fn test_ttl_expiration() {
        let store = Store::new();
        store.insert(
            "key2".to_string(),
            "value2".to_string(),
            Some(Duration::from_millis(100)),
        );

        std::thread::sleep(Duration::from_millis(101));

        assert_eq!(store.get("key2"), None);
    }
}
