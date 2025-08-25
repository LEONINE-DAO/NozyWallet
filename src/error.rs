//! Error handling for the Nozy wallet

use thiserror::Error;


pub type NozyResult<T> = Result<T, NozyError>;


#[derive(Error, Debug, Clone)]
pub enum NozyError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Note error: {0}")]
    Note(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),
}

impl From<std::io::Error> for NozyError {
    fn from(err: std::io::Error) -> Self {
        NozyError::Storage(err.to_string())
    }
}

impl From<serde_json::Error> for NozyError {
    fn from(err: serde_json::Error) -> Self {
        NozyError::Storage(format!("JSON error: {}", err))
    }
}

impl From<bip39::Error> for NozyError {
    fn from(err: bip39::Error) -> Self {
        NozyError::InvalidOperation(format!("BIP39 error: {}", err))
    }
}

impl From<bip32::Error> for NozyError {
    fn from(err: bip32::Error) -> Self {
        NozyError::InvalidOperation(format!("BIP32 error: {}", err))
    }
} 
