

use crate::error::{NozyError, NozyResult};
use bip39::Mnemonic;
use bip32::{DerivationPath, XPrv};
use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use rand::Rng;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HDWallet {
    
    pub seed_phrase: Option<String>,
    
    
    pub seed_hash: Option<String>,
    
    
    pub encrypted_master_key: Option<EncryptedKey>,
    
    
    pub derived_addresses: HashMap<String, DerivedAddress>,
    
    
    pub network: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    
    pub encrypted_data: Vec<u8>,
    
    pub nonce: Vec<u8>,
    
    pub salt: Vec<u8>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedAddress {
    
    pub path: String,
    
    
    pub address_type: AddressType,
    
    
    pub address: String,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AddressType {
    
    Orchard,
    
    Sapling,
    
    Unified,
}

impl HDWallet {
    
    pub fn new_from_seed(seed_phrase: &str, network: &str) -> NozyResult<Self> {
        // Validate seed phrase
        let mnemonic = Mnemonic::parse_normalized(seed_phrase)?;
        
        // Generate seed from mnemonic
        let seed = mnemonic.to_seed("");
        
        // Create master private key using BIP32
        let master_key = XPrv::new(&seed)?;
        
        // Encrypt the master key with a default password (will be changed by user)
        let encrypted_master_key = Some(Self::encrypt_key(&master_key.to_bytes(), "default_password")?);
        
        // Generate seed hash for verification
        let seed_hash = Self::hash_seed(seed_phrase);
        
        Ok(Self {
            seed_phrase: Some(seed_phrase.to_string()),
            seed_hash: Some(seed_hash),
            encrypted_master_key,
            derived_addresses: HashMap::new(),
            network: network.to_string(),
        })
    }
    
    
    pub fn generate_seed() -> NozyResult<String> {
        // Generate 128 bits of entropy (12 words)
        let entropy = rand::random::<[u8; 16]>();
        let mnemonic = Mnemonic::from_entropy(&entropy)?;
        
        Ok(mnemonic.to_string())
    }
    
    
    pub fn verify_seed(&self, seed_phrase: &str) -> bool {
        if let Some(stored_hash) = &self.seed_hash {
            let input_hash = Self::hash_seed(seed_phrase);
            stored_hash == &input_hash
        } else {
            false
        }
    }
    
    
    pub fn derive_address(&mut self, path: &str, address_type: AddressType) -> NozyResult<DerivedAddress> {
        // Check if already derived
        if let Some(existing) = self.derived_addresses.get(path) {
            if existing.address_type == address_type {
                return Ok(existing.clone());
            }
        }
        
        // Get the master key for derivation (using default password for now)
        let master_key = self.get_master_key("default_password")?;
        
        // Parse the derivation path
        let derivation_path = DerivationPath::from_str(path)?;
        
        // Derive the child key step by step
        let mut current_key = master_key;
        for child_number in derivation_path.iter() {
            current_key = current_key.derive_child(child_number)?;
        }
        
        // Generate the address based on the derived key
        let address = match address_type {
            AddressType::Orchard => format!("o{}", Self::generate_address_from_key(&current_key, "orchard")),
            AddressType::Sapling => format!("z{}", Self::generate_address_from_key(&current_key, "sapling")),
            AddressType::Unified => format!("u{}", Self::generate_address_from_key(&current_key, "unified")),
        };
        
        let derived_address = DerivedAddress {
            path: path.to_string(),
            address_type,
            address,
        };
        
        // Cache the derived address
        self.derived_addresses.insert(path.to_string(), derived_address.clone());
        
        Ok(derived_address)
    }
    
    
    pub fn get_seed_phrase(&self) -> Option<&String> {
        self.seed_phrase.as_ref()
    }
    
    
    pub fn get_seed_hash(&self) -> Option<&String> {
        self.seed_hash.as_ref()
    }
    
    
    pub fn get_derived_addresses(&self) -> &HashMap<String, DerivedAddress> {
        &self.derived_addresses
    }
    
    
    fn hash_seed(seed_phrase: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seed_phrase.as_bytes());
        hex::encode(hasher.finalize())
    }
    
    
    pub fn get_master_key(&self, password: &str) -> NozyResult<XPrv> {
        let encrypted_key = self.encrypted_master_key.as_ref()
            .ok_or_else(|| NozyError::InvalidOperation("No master key found".to_string()))?;
        
        let key_bytes = Self::decrypt_key(encrypted_key, password)?;
        let master_key = XPrv::new(&key_bytes)?;
        Ok(master_key)
    }
    
    
    fn generate_address_from_key(key: &XPrv, key_type: &str) -> String {
        // For now, generate a deterministic address based on the key
        // TODO: Implement actual Zcash address generation
        let public_key = key.public_key().to_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&public_key);
        hasher.update(key_type.as_bytes());
        hex::encode(&hasher.finalize()[..16])
    }
    
    
    fn encrypt_key(key_data: &[u8], password: &str) -> NozyResult<EncryptedKey> {
        // Generate random salt and nonce
        let mut rng = rand::thread_rng();
        let salt: [u8; 32] = rng.gen();
        let nonce: [u8; 12] = rng.gen();
        
        // Derive encryption key from password and salt
        let encryption_key = Self::derive_encryption_key(password, &salt)?;
        
        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new(&encryption_key);
        
        // Convert nonce to proper type for AES-GCM
        let nonce_ref = Nonce::from_slice(&nonce);
        
        // Encrypt the key data
        let encrypted_data = cipher.encrypt(nonce_ref, key_data)
            .map_err(|e| NozyError::InvalidOperation(format!("Encryption failed: {}", e)))?;
        
        Ok(EncryptedKey {
            encrypted_data,
            nonce: nonce.to_vec(),
            salt: salt.to_vec(),
        })
    }
    
    
    fn decrypt_key(encrypted_key: &EncryptedKey, password: &str) -> NozyResult<Vec<u8>> {
        // Derive encryption key from password and salt
        let encryption_key = Self::derive_encryption_key(password, &encrypted_key.salt)?;
        
        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new(&encryption_key);
        
        // Convert Vec<u8> to proper types for AES-GCM
        let nonce_array: [u8; 12] = encrypted_key.nonce.as_slice().try_into()
            .map_err(|_| NozyError::InvalidOperation("Invalid nonce length".to_string()))?;
        let nonce = Nonce::from_slice(&nonce_array);
        
        // Decrypt the key data
        let decrypted_data = cipher.decrypt(nonce, &*encrypted_key.encrypted_data)
            .map_err(|e| NozyError::InvalidOperation(format!("Decryption failed: {}", e)))?;
        
        Ok(decrypted_data)
    }
    
    
    fn derive_encryption_key(password: &str, salt: &[u8]) -> NozyResult<Key<Aes256Gcm>> {
        // Use PBKDF2 to derive key from password
        let mut key = [0u8; 32];
        pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
            password.as_bytes(),
            salt,
            100_000, // 100k iterations for security
            &mut key
        );
        
        Ok(Key::<Aes256Gcm>::from_slice(&key).clone())
    }
    
    
    fn generate_placeholder_address(path: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        hex::encode(&hasher.finalize()[..16])
    }
    
    
    pub fn get_change_address(&self) -> NozyResult<String> {
        // Generate change address using a specific derivation path
        let change_path = "m/44'/133'/0'/1/0"; // Change address path
        let change_address = Self::generate_placeholder_address(change_path);
        Ok(format!("o{}", change_address)) // Orchard change address
    }
    
    
    pub fn get_seed_bytes(&self, password: &str) -> NozyResult<Vec<u8>> {
        let mnemonic = Mnemonic::parse_normalized(
            self.seed_phrase.as_ref()
                .ok_or_else(|| NozyError::InvalidOperation("No seed phrase found".to_string()))?
        )?;
        
        // Generate seed from mnemonic (empty passphrase for now)
        let seed = mnemonic.to_seed("");
        Ok(seed.to_vec())
    }
    
    
    pub fn derive_child_key(&self, derivation_path: &str, password: &str) -> NozyResult<Vec<u8>> {
        // Get raw seed bytes for proper Zcash derivation
        let seed = self.get_seed_bytes(password)?;
        
        // Parse derivation path
        let path = bip32::DerivationPath::from_str(derivation_path)
            .map_err(|e| NozyError::InvalidOperation(format!("Invalid derivation path: {}", e)))?;
        
        // For now, use a simplified approach that's compatible with our current setup
        // In production, this would use proper Zcash key derivation
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        hasher.update(derivation_path.as_bytes());
        let child_key = hasher.finalize().to_vec();
        
        Ok(child_key)
    }
}

impl Default for HDWallet {
    fn default() -> Self {
        Self {
            seed_phrase: None,
            seed_hash: None,
            encrypted_master_key: None,
            derived_addresses: HashMap::new(),
            network: "testnet".to_string(),
        }
    }
} 
