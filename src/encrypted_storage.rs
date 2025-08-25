use crate::error::{NozyResult, NozyError};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha256;
use serde::{Serialize, Deserialize};
use serde_json;
use std::path::{Path, PathBuf};
use std::fs;
use rand::Rng;


pub struct EncryptedStorage {
    
    storage_dir: PathBuf,
    
    master_key: Option<Vec<u8>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedFile {
    
    pub encrypted_data: Vec<u8>,
    
    pub nonce: Vec<u8>,
    
    pub salt: Vec<u8>,
    
    pub version: u32,
}

impl EncryptedStorage {
    
    pub fn new(storage_dir: &Path) -> NozyResult<Self> {
        // Create storage directory if it doesn't exist
        fs::create_dir_all(storage_dir)
            .map_err(|e| NozyError::Storage(format!("Failed to create storage directory: {}", e)))?;
        
        Ok(Self {
            storage_dir: storage_dir.to_path_buf(),
            master_key: None,
        })
    }
    
    
    pub fn initialize(&mut self, password: &str) -> NozyResult<()> {
        // Generate random salt for this storage instance
        let mut rng = rand::thread_rng();
        let salt: [u8; 32] = rng.gen();
        
        // Derive master encryption key from password
        let mut key = [0u8; 32];
        pbkdf2::<Hmac<Sha256>>(
            password.as_bytes(),
            &salt,
            100_000, // 100k iterations for security
            &mut key
        );
        
        self.master_key = Some(key.to_vec());
        Ok(())
    }
    
    
    pub fn is_initialized(&self) -> bool {
        self.master_key.is_some()
    }
    
    
    pub fn save_encrypted<T: Serialize>(&self, filename: &str, data: &T) -> NozyResult<()> {
        let master_key = self.master_key.as_ref()
            .ok_or_else(|| NozyError::InvalidOperation("Storage not initialized".to_string()))?;
        
        // Serialize data to JSON
        let json_data = serde_json::to_vec(data)
            .map_err(|e| NozyError::Serialization(format!("Failed to serialize data: {}", e)))?;
        
        // Encrypt the data
        let encrypted_file = Self::encrypt_data(&json_data, master_key)?;
        
        // Save encrypted file
        let file_path = self.storage_dir.join(format!("{}.enc", filename));
        let encrypted_bytes = serde_json::to_vec(&encrypted_file)
            .map_err(|e| NozyError::Serialization(format!("Failed to serialize encrypted file: {}", e)))?;
        
        fs::write(file_path, encrypted_bytes)
            .map_err(|e| NozyError::Storage(format!("Failed to write encrypted file: {}", e)))?;
        
        Ok(())
    }
    
    
    pub fn load_encrypted<T: for<'de> Deserialize<'de>>(&self, filename: &str) -> NozyResult<T> {
        let master_key = self.master_key.as_ref()
            .ok_or_else(|| NozyError::InvalidOperation("Storage not initialized".to_string()))?;
        
        // Read encrypted file
        let file_path = self.storage_dir.join(format!("{}.enc", filename));
        let encrypted_bytes = fs::read(file_path)
            .map_err(|e| NozyError::Storage(format!("Failed to read encrypted file: {}", e)))?;
        
        // Deserialize encrypted file metadata
        let encrypted_file: EncryptedFile = serde_json::from_slice(&encrypted_bytes)
            .map_err(|e| NozyError::Serialization(format!("Failed to deserialize encrypted file: {}", e)))?;
        
        // Decrypt the data
        let decrypted_data = Self::decrypt_data(&encrypted_file, master_key)?;
        
        // Deserialize the decrypted JSON data
        let data: T = serde_json::from_slice(&decrypted_data)
            .map_err(|e| NozyError::Serialization(format!("Failed to deserialize decrypted data: {}", e)))?;
        
        Ok(data)
    }
    
    
    pub fn create_backup(&self, backup_path: &Path) -> NozyResult<()> {
        let master_key = self.master_key.as_ref()
            .ok_or_else(|| NozyError::InvalidOperation("Storage not initialized".to_string()))?;
        
        // Create backup directory
        fs::create_dir_all(backup_path)
            .map_err(|e| NozyError::Storage(format!("Failed to create backup directory: {}", e)))?;
        
        // Copy all encrypted files to backup location
        for entry in fs::read_dir(&self.storage_dir)
            .map_err(|e| NozyError::Storage(format!("Failed to read storage directory: {}", e)))? {
            let entry = entry
                .map_err(|e| NozyError::Storage(format!("Failed to read directory entry: {}", e)))?;
            
            if entry.path().extension().map_or(false, |ext| ext == "enc") {
                let filename = entry.file_name();
                let backup_file = backup_path.join(filename);
                
                fs::copy(entry.path(), backup_file)
                    .map_err(|e| NozyError::Storage(format!("Failed to copy file to backup: {}", e)))?;
            }
        }
        
        Ok(())
    }
    
    
    pub fn restore_from_backup(&mut self, backup_path: &Path, password: &str) -> NozyResult<()> {
        // Initialize storage with password
        self.initialize(password)?;
        
        // Clear existing storage
        for entry in fs::read_dir(&self.storage_dir)
            .map_err(|e| NozyError::Storage(format!("Failed to read storage directory: {}", e)))? {
            let entry = entry
                .map_err(|e| NozyError::Storage(format!("Failed to read directory entry: {}", e)))?;
            
            if entry.path().extension().map_or(false, |ext| ext == "enc") {
                fs::remove_file(entry.path())
                    .map_err(|e| NozyError::Storage(format!("Failed to remove existing file: {}", e)))?;
            }
        }
        
        // Copy backup files to storage
        for entry in fs::read_dir(backup_path)
            .map_err(|e| NozyError::Storage(format!("Failed to read backup directory: {}", e)))? {
            let entry = entry
                .map_err(|e| NozyError::Storage(format!("Failed to read backup entry: {}", e)))?;
            
            if entry.path().extension().map_or(false, |ext| ext == "enc") {
                let filename = entry.file_name();
                let storage_file = self.storage_dir.join(filename);
                
                fs::copy(entry.path(), storage_file)
                    .map_err(|e| NozyError::Storage(format!("Failed to copy backup file: {}", e)))?;
            }
        }
        
        Ok(())
    }
    
    
    pub fn list_files(&self) -> NozyResult<Vec<String>> {
        let mut files = Vec::new();
        
        for entry in fs::read_dir(&self.storage_dir)
            .map_err(|e| NozyError::Storage(format!("Failed to read storage directory: {}", e)))? {
            let entry = entry
                .map_err(|e| NozyError::Storage(format!("Failed to read directory entry: {}", e)))?;
            
            if entry.path().extension().map_or(false, |ext| ext == "enc") {
                if let Some(filename) = entry.file_name().to_str() {
                    // Remove .enc extension for display
                    let name = filename.trim_end_matches(".enc");
                    files.push(name.to_string());
                }
            }
        }
        
        Ok(files)
    }
    
    
    pub fn file_exists(&self, filename: &str) -> bool {
        let file_path = self.storage_dir.join(format!("{}.enc", filename));
        file_path.exists()
    }
    
    
    pub fn delete_file(&self, filename: &str) -> NozyResult<()> {
        let file_path = self.storage_dir.join(format!("{}.enc", filename));
        
        if file_path.exists() {
            fs::remove_file(file_path)
                .map_err(|e| NozyError::Storage(format!("Failed to delete file: {}", e)))?;
        }
        
        Ok(())
    }
    
    
    fn encrypt_data(data: &[u8], key: &[u8]) -> NozyResult<EncryptedFile> {
        // Generate random salt and nonce
        let mut rng = rand::thread_rng();
        let salt: [u8; 32] = rng.gen();
        let nonce: [u8; 12] = rng.gen();
        
        // Derive encryption key from master key and salt
        let mut derived_key = [0u8; 32];
        pbkdf2::<Hmac<Sha256>>(
            key,
            &salt,
            10_000, // 10k iterations for file encryption
            &mut derived_key
        );
        
        // Create AES-256-GCM cipher
        let encryption_key = Key::<Aes256Gcm>::from_slice(&derived_key).clone();
        let cipher = Aes256Gcm::new(&encryption_key);
        
        // Encrypt the data
        let nonce_ref = Nonce::from_slice(&nonce);
        let encrypted_data = cipher.encrypt(nonce_ref, data)
            .map_err(|e| NozyError::InvalidOperation(format!("File encryption failed: {}", e)))?;
        
        Ok(EncryptedFile {
            encrypted_data,
            nonce: nonce.to_vec(),
            salt: salt.to_vec(),
            version: 1,
        })
    }
    
    
    fn decrypt_data(encrypted_file: &EncryptedFile, key: &[u8]) -> NozyResult<Vec<u8>> {
        // Derive decryption key from master key and salt
        let mut derived_key = [0u8; 32];
        pbkdf2::<Hmac<Sha256>>(
            key,
            &encrypted_file.salt,
            10_000, // 10k iterations for file decryption
            &mut derived_key
        );
        
        // Create AES-256-GCM cipher
        let decryption_key = Key::<Aes256Gcm>::from_slice(&derived_key).clone();
        let cipher = Aes256Gcm::new(&decryption_key);
        
        // Convert nonce to proper type
        let nonce_array: [u8; 12] = encrypted_file.nonce.as_slice().try_into()
            .map_err(|_| NozyError::InvalidOperation("Invalid nonce length".to_string()))?;
        let nonce = Nonce::from_slice(&nonce_array);
        
        // Decrypt the data
        let decrypted_data = cipher.decrypt(nonce, &*encrypted_file.encrypted_data)
            .map_err(|e| NozyError::InvalidOperation(format!("File decryption failed: {}", e)))?;
        
        Ok(decrypted_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_encrypted_storage() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("wallet_storage");
        
        let mut storage = EncryptedStorage::new(&storage_path).unwrap();
        storage.initialize("test_password").unwrap();
        
        // Test saving and loading encrypted data
        let test_data = vec!["test1", "test2", "test3"];
        storage.save_encrypted("test_file", &test_data).unwrap();
        
        let loaded_data: Vec<&str> = storage.load_encrypted("test_file").unwrap();
        assert_eq!(test_data, loaded_data);
        
        // Test file listing
        let files = storage.list_files().unwrap();
        assert!(files.contains(&"test_file".to_string()));
        
        // Test file existence
        assert!(storage.file_exists("test_file"));
        
        // Test file deletion
        storage.delete_file("test_file").unwrap();
        assert!(!storage.file_exists("test_file"));
    }
    
    #[test]
    fn test_backup_and_restore() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("wallet_storage");
        let backup_path = temp_dir.path().join("backup");
        
        let mut storage = EncryptedStorage::new(&storage_path).unwrap();
        storage.initialize("test_password").unwrap();
        
        // Save some test data
        let test_data = vec!["backup_test"];
        storage.save_encrypted("backup_file", &test_data).unwrap();
        
        // Create backup
        storage.create_backup(&backup_path).unwrap();
        
        // Create new storage and restore from backup
        let mut new_storage = EncryptedStorage::new(&temp_dir.path().join("new_storage")).unwrap();
        new_storage.restore_from_backup(&backup_path, "test_password").unwrap();
        
        // Verify data was restored
        let restored_data: Vec<&str> = new_storage.load_encrypted("backup_file").unwrap();
        assert_eq!(test_data, restored_data);
    }
} 
