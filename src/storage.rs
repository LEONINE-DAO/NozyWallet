use crate::error::NozyResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStorage {
    data: HashMap<String, Vec<u8>>,
}

impl WalletStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn store(&mut self, key: &str, value: &[u8]) -> NozyResult<()> {
        self.data.insert(key.to_string(), value.to_vec());
        Ok(())
    }
    
    pub fn retrieve(&self, key: &str) -> NozyResult<Option<Vec<u8>>> {
        Ok(self.data.get(key).cloned())
    }
    
    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    pub fn remove(&mut self, key: &str) -> NozyResult<()> {
        self.data.remove(key);
        Ok(())
    }
    
    pub fn get_all_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}