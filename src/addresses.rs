
use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NetworkType {
    Mainnet,
    Testnet,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ZcashAddressType {
    
    Orchard,
    
    Sapling,
    
    Unified,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ZcashAddressWrapper {
    pub address: String,
    pub address_type: ZcashAddressType,
    pub derivation_path: String,
    pub network: NetworkType,
}

impl ZcashAddressWrapper {
    
    pub fn new(address: String, address_type: ZcashAddressType, derivation_path: String, network: NetworkType) -> Self {
        Self {
            address,
            address_type,
            derivation_path,
            network,
        }
    }

    
    pub fn validate_address(&self, address: &str) -> bool {
        
        if address.starts_with("u") && address.len() >= 50 && address.len() <= 70 {
            return hex::decode(&address[1..]).is_ok();
        }
        
        if address.starts_with("z") && address.len() >= 50 && address.len() <= 70 {
            return hex::decode(&address[1..]).is_ok();
        }
        
        false
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressManager {
    
    addresses: HashMap<String, ZcashAddressWrapper>,
    
    
    counters: HashMap<ZcashAddressType, u32>,
    
    
    hd_wallet: HDWallet,
    
    
    network: NetworkType,
}

impl AddressManager {
    
    pub fn new(hd_wallet: HDWallet, network: NetworkType) -> Self {
        Self {
            addresses: HashMap::new(),
            counters: HashMap::new(),
            hd_wallet,
            network,
        }
    }
    
    
    pub fn generate_orchard_address(&mut self, password: &str) -> NozyResult<ZcashAddressWrapper> {
        let counter_value = *self.counters.entry(ZcashAddressType::Orchard).or_insert(0);
        let derivation_path = format!("m/44'/133'/0'/0/{}", counter_value);
        
        let seed = self.hd_wallet.get_seed_bytes(password)?;
        
        
        let address_string = self.generate_orchard_address_string(&seed, counter_value)?;
        
        let zcash_address = ZcashAddressWrapper::new(
            address_string,
            ZcashAddressType::Orchard,
            derivation_path.clone(),
            self.network,
        );
        
        self.addresses.insert(zcash_address.address.clone(), zcash_address.clone());
        *self.counters.get_mut(&ZcashAddressType::Orchard).unwrap() += 1;
        
        Ok(zcash_address)
    }
    
    
    pub fn generate_sapling_address(&mut self, password: &str) -> NozyResult<ZcashAddressWrapper> {
        let counter_value = *self.counters.entry(ZcashAddressType::Sapling).or_insert(0);
        let derivation_path = format!("m/44'/133'/0'/0/{}", counter_value);
        
        let seed = self.hd_wallet.get_seed_bytes(password)?;
        
        let address_string = self.generate_sapling_address_string(&seed, counter_value)?;
        
        let zcash_address = ZcashAddressWrapper::new(
            address_string,
            ZcashAddressType::Sapling,
            derivation_path.clone(),
            self.network,
        );
        
        self.addresses.insert(zcash_address.address.clone(), zcash_address.clone());
        *self.counters.get_mut(&ZcashAddressType::Sapling).unwrap() += 1;
        
        Ok(zcash_address)
    }
    
    
    pub fn generate_unified_address(&mut self, password: &str) -> NozyResult<ZcashAddressWrapper> {
        let counter_value = *self.counters.entry(ZcashAddressType::Unified).or_insert(0);
        let derivation_path = format!("m/44'/133'/0'/0/{}", counter_value);
        
        let seed = self.hd_wallet.get_seed_bytes(password)?;
        
        let address_string = self.generate_unified_address_string(&seed, counter_value)?;
        
        let zcash_address = ZcashAddressWrapper::new(
            address_string,
            ZcashAddressType::Unified,
            derivation_path.clone(),
            self.network,
        );
        
        self.addresses.insert(zcash_address.address.clone(), zcash_address.clone());
        *self.counters.get_mut(&ZcashAddressType::Unified).unwrap() += 1;
        
        Ok(zcash_address)
    }
    
    
    fn generate_orchard_address_string(&self, seed: &[u8], counter: u32) -> NozyResult<String> {
        use blake2b_simd::Params;
        
        let mut hasher = Params::new()
            .hash_length(32)
            .to_state();
        
        hasher.update(b"Orchard_Address");
        hasher.update(seed);
        hasher.update(&counter.to_le_bytes());
        hasher.update(&self.network.to_string().as_bytes());
        
        let hash = hasher.finalize();
        
        let address = format!("u{}", hex::encode(&hash.as_bytes()[..28]));
        
        Ok(address)
    }
    
    
    fn generate_sapling_address_string(&self, seed: &[u8], counter: u32) -> NozyResult<String> {
        use blake2b_simd::Params;
        
        let mut hasher = Params::new()
            .hash_length(32)
            .to_state();
        
        hasher.update(b"Sapling_Address");
        hasher.update(seed);
        hasher.update(&counter.to_le_bytes());
        hasher.update(&self.network.to_string().as_bytes());
        
        let hash = hasher.finalize();
        
        let address = format!("z{}", hex::encode(&hash.as_bytes()[..28]));
        
        Ok(address)
    }
    
    
    fn generate_unified_address_string(&self, seed: &[u8], counter: u32) -> NozyResult<String> {
        use blake2b_simd::Params;
        
        let mut hasher = Params::new()
            .hash_length(32)
            .to_state();
        
        hasher.update(b"Unified_Address");
        hasher.update(seed);
        hasher.update(&counter.to_le_bytes());
        hasher.update(&self.network.to_string().as_bytes());
        
        let hash = hasher.finalize();
        
        let address = format!("u{}", hex::encode(&hash.as_bytes()[..28]));
        
        Ok(address)
    }
    
    
    pub fn get_all_addresses(&self) -> Vec<&ZcashAddressWrapper> {
        self.addresses.values().collect()
    }
    
    
    pub fn get_addresses_by_type(&self, address_type: &ZcashAddressType) -> Vec<&ZcashAddressWrapper> {
        self.addresses.values()
            .filter(|addr| addr.address_type == *address_type)
            .collect()
    }
    
    
    pub fn find_address(&self, address: &str) -> Option<&ZcashAddressWrapper> {
        self.addresses.get(address)
    }
    
    
    pub fn validate_address(&self, address: &str) -> bool {
        
        if address.starts_with("u") && address.len() >= 50 && address.len() <= 70 {
            return hex::decode(&address[1..]).is_ok();
        }
        
        if address.starts_with("z") && address.len() >= 50 && address.len() <= 70 {
            return hex::decode(&address[1..]).is_ok();
        }
        
        false
    }
    
    
    pub fn get_address_count(&self, address_type: &ZcashAddressType) -> u32 {
        *self.counters.get(address_type).unwrap_or(&0)
    }
    
    
    pub fn import_address(&mut self, address: ZcashAddressWrapper) -> NozyResult<()> {
        if !self.validate_address(&address.address) {
            return Err(NozyError::InvalidOperation("Invalid Zcash address format".to_string()));
        }
        
        self.addresses.insert(address.address.clone(), address);
        Ok(())
    }
    
    
    pub fn get_network(&self) -> NetworkType {
        self.network
    }
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkType::Mainnet => write!(f, "mainnet"),
            NetworkType::Testnet => write!(f, "testnet"),
        }
    }
} 
