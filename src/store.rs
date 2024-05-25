use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Store {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, key: String, value: String) {
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    pub fn remove(&self, key: &str) -> Option<String> {
        let mut data = self.data.write().unwrap();
        data.remove(key)
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Store {
            data: self.data.clone(),
        }
    }
}
