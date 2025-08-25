//! Configuration for the Nozy wallet

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NozyConfig {
    
    pub network: NetworkConfig,
    
    
    pub privacy: PrivacyConfig,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    
    pub network: String,
    
    
    pub default_privacy: PrivacyLevel,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    
    pub enable_orchard: bool,
    
    
    pub enable_sapling: bool,
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    
    Maximum,
    
    
    High,
    
    
    Balanced,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrivacyMaskType {
    
    Random,

    
    FakeSpending,

    
    Noise,

    
    Custom,
}

impl From<PrivacyMaskType> for crate::cli::PrivacyMaskType {
    fn from(mask_type: PrivacyMaskType) -> Self {
        match mask_type {
            PrivacyMaskType::Random => crate::cli::PrivacyMaskType::Random,
            PrivacyMaskType::FakeSpending => crate::cli::PrivacyMaskType::FakeSpending,
            PrivacyMaskType::Noise => crate::cli::PrivacyMaskType::Noise,
            PrivacyMaskType::Custom => crate::cli::PrivacyMaskType::Custom,
        }
    }
}

impl From<crate::cli::PrivacyMaskType> for PrivacyMaskType {
    fn from(mask_type: crate::cli::PrivacyMaskType) -> Self {
        match mask_type {
            crate::cli::PrivacyMaskType::Random => PrivacyMaskType::Random,
            crate::cli::PrivacyMaskType::FakeSpending => PrivacyMaskType::FakeSpending,
            crate::cli::PrivacyMaskType::Noise => PrivacyMaskType::Noise,
            crate::cli::PrivacyMaskType::Custom => PrivacyMaskType::Custom,
        }
    }
}

impl NozyConfig {
    
    pub fn new(privacy_level: PrivacyLevel) -> Self {
        Self {
            network: NetworkConfig {
                network: "mainnet".to_string(),
                default_privacy: privacy_level,
            },
            privacy: PrivacyConfig {
                enable_orchard: privacy_level == PrivacyLevel::Maximum,
                enable_sapling: privacy_level != PrivacyLevel::Balanced,
            },
        }
    }
}

impl Default for NozyConfig {
    fn default() -> Self {
        Self::new(PrivacyLevel::Maximum)
    }
} 


// some people ware masks becuase they shame of their spending habits somw becuase they vale privacy.
