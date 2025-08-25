//Nozy is a privacy game changer we are tam Zebrad built fully private and secure Nozy wallet

use crate::error::NozyResult;
use crate::config::PrivacyLevel;
use crate::notes::{ShieldedNote, NoteType};
use crate::addresses::ZcashAddressWrapper;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub note: ShieldedNote,
    pub nullifier: String,
    pub witness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub address: ZcashAddressWrapper,
    pub amount: u64,
    pub note_type: NoteType,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedTransaction {
    pub txid: String,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub fee: u64,
    pub privacy_level: PrivacyLevel,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    Building,
    Ready,
    Broadcast,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBuilder {
    current_transaction: Option<ShieldedTransaction>,
    default_privacy: PrivacyLevel,
}

impl TransactionBuilder {
    pub fn new(default_privacy: PrivacyLevel) -> Self {
        Self {
            current_transaction: None,
            default_privacy,
        }
    }
    
    pub fn start_transaction(&mut self, privacy_level: Option<PrivacyLevel>) -> NozyResult<()> {
        let privacy = privacy_level.unwrap_or(self.default_privacy);
        
        self.current_transaction = Some(ShieldedTransaction {
            txid: format!("tx_{}", chrono::Utc::now().timestamp()),
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee: 0,
            privacy_level: privacy,
            status: TransactionStatus::Building,
        });
        
        Ok(())
    }
    
    pub fn add_input(&mut self, note: ShieldedNote) -> NozyResult<()> {
        if let Some(tx) = &mut self.current_transaction {
            let input = TransactionInput {
                note: note.clone(),
                nullifier: format!("null_{}", note.id),
                witness: format!("witness_{}", note.id),
            };
            tx.inputs.push(input);
            Ok(())
        } else {
            Err(crate::error::NozyError::InvalidOperation(
                "No transaction in progress".to_string()
            ))
        }
    }
    
    pub fn add_output(&mut self, address: ZcashAddressWrapper, amount: u64, note_type: NoteType) -> NozyResult<()> {
        if let Some(tx) = &mut self.current_transaction {
            let output = TransactionOutput {
                address,
                amount,
                note_type,
                memo: None,
            };
            tx.outputs.push(output);
            Ok(())
        } else {
            Err(crate::error::NozyError::InvalidOperation(
                "No transaction in progress".to_string()
            ))
        }
    }
    
    pub fn set_fee(&mut self, fee: u64) -> NozyResult<()> {
        if let Some(tx) = &mut self.current_transaction {
            tx.fee = fee;
            Ok(())
        } else {
            Err(crate::error::NozyError::InvalidOperation(
                "No transaction in progress".to_string()
            ))
        }
    }
    
    pub fn get_current_transaction(&self) -> Option<&ShieldedTransaction> {
        self.current_transaction.as_ref()
    }
    
    pub fn finalize(&mut self) -> NozyResult<ShieldedTransaction> {
        if let Some(mut tx) = self.current_transaction.take() {
            tx.status = TransactionStatus::Ready;
            Ok(tx)
        } else {
            Err(crate::error::NozyError::InvalidOperation(
                "No transaction in progress".to_string()
            ))
        }
    }
} 