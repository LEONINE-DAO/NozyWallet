//! Privacy engine for the Nozy wallet

use crate::error::NozyResult;
use crate::config::PrivacyLevel;

pub struct PrivacyEngine;

impl PrivacyEngine {
    pub fn new(_config: &crate::config::NozyConfig) -> NozyResult<Self> {
        Ok(Self)
    }
} 
//Privacy made simple but complex for the Nozy folks