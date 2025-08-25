
// Nozy is the best wallet in the world we are team Zebrad built fully private and secure Nozy wallet

use crate::error::{NozyResult, NozyError};
use crate::hd_wallet::{HDWallet, AddressType};
use crate::notes::{NoteManager, ShieldedNote, NoteType, NoteSelectionStrategy};
use serde::{Serialize, Deserialize};
use ed25519_dalek::{Signer, Verifier, Signature, SigningKey, VerifyingKey};
use blake2b_simd::Params;
use sha2::Digest;
use std::collections::HashMap;


pub struct TransactionSigner {
    
    hd_wallet: HDWallet,
    
    signing_keys: HashMap<String, SigningKey>,
    
    note_manager: NoteManager,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedInput {
    
    pub note: ShieldedNote,
    
    pub merkle_path: Vec<Vec<u8>>,
    
    pub position: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedOutput {
    
    pub address: String,
    
    pub value: u64,
    
    pub memo: Option<Vec<u8>>,
    
    pub address_type: AddressType,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
    
    pub signature: Vec<u8>,
    
    pub public_key: Vec<u8>,
    
    pub algorithm: SignatureAlgorithm,
    
    pub tx_hash: Vec<u8>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    
    RedPallas,
    
    RedJubjub,
    
    EdDSA,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    
    pub inputs: Vec<ShieldedInput>,
    
    pub outputs: Vec<ShieldedOutput>,
    
    pub fee: u64,
    
    pub signatures: Vec<TransactionSignature>,
    
    pub tx_hash: Vec<u8>,
    
    pub expiry_height: u64,
    
    pub version: u32,
    
    pub change_output: Option<ShieldedOutput>,
}

impl TransactionSigner {
    
    pub fn new(hd_wallet: HDWallet, note_manager: NoteManager) -> Self {
        Self {
            hd_wallet,
            signing_keys: HashMap::new(),
            note_manager,
        }
    }
    
    
    pub fn build_transaction_with_notes(
        &mut self,
        recipient_address: String,
        amount: u64,
        fee: u64,
        memo: Option<Vec<u8>>,
        expiry_height: u64,
        strategy: Option<NoteSelectionStrategy>,
    ) -> NozyResult<SignedTransaction> {
        // Calculate total amount needed (including fee)
        let total_needed = amount + fee;
        
        // Select notes to spend based on strategy
        let notes_to_spend = self.note_manager.select_notes_for_spending(
            total_needed,
            strategy,
        )?;
        
        // Calculate total input value
        let total_input: u64 = notes_to_spend.iter().map(|note| note.value).sum();
        
        // Calculate change amount
        let change_amount = total_input - total_needed;
        
        // Create inputs from selected notes
        let inputs: Vec<ShieldedInput> = notes_to_spend.iter().map(|note| {
            ShieldedInput {
                note: (*note).clone(),
                merkle_path: note.merkle_path.clone().unwrap_or_default(),
                position: note.position.unwrap_or(0),
            }
        }).collect();
        
        // Create outputs
        let mut outputs = vec![ShieldedOutput {
            address: recipient_address,
            value: amount,
            memo,
            address_type: AddressType::Orchard, // Default to Orchard for privacy
        }];
        
        // Add change output if needed
        let change_output = if change_amount > 0 {
            let change = ShieldedOutput {
                address: self.hd_wallet.get_change_address()?, // Get change address
                value: change_amount,
                memo: None, // No memo for change
                address_type: AddressType::Orchard,
            };
            outputs.push(change.clone());
            Some(change)
        } else {
            None
        };
        
        // Calculate transaction hash
        let tx_hash = self.calculate_transaction_hash(&inputs, &outputs, fee, expiry_height)?;
        
        // Create unsigned transaction
        let transaction = SignedTransaction {
            inputs,
            outputs,
            fee,
            signatures: Vec::new(), // Will be filled when signing
            tx_hash,
            expiry_height,
            version: 5, // Zcash v5 (latest)
            change_output,
        };
        
        Ok(transaction)
    }
    
    
    fn calculate_transaction_hash(
        &self,
        inputs: &[ShieldedInput],
        outputs: &[ShieldedOutput],
        fee: u64,
        expiry_height: u64,
    ) -> NozyResult<Vec<u8>> {
        let mut hasher = Params::new()
            .hash_length(32)
            .to_state();
        
        // Hash inputs
        for input in inputs {
            hasher.update(&input.note.commitment);
            hasher.update(&input.position.to_le_bytes());
        }
        
        // Hash outputs
        for output in outputs {
            hasher.update(output.address.as_bytes());
            hasher.update(&output.value.to_le_bytes());
            if let Some(ref memo) = output.memo {
                hasher.update(memo);
            }
        }
        
        // Hash fee and expiry
        hasher.update(&fee.to_le_bytes());
        hasher.update(&expiry_height.to_le_bytes());
        
        Ok(hasher.finalize().as_bytes().to_vec())
    }
    
    
    pub fn sign_transaction_with_notes(
        &mut self,
        mut transaction: SignedTransaction,
        password: &str,
    ) -> NozyResult<SignedTransaction> {
        let mut signatures = Vec::new();
        
        // Sign each input
        for (i, input) in transaction.inputs.iter().enumerate() {
            let derivation_path = self.get_derivation_path_for_note(&input.note)?;
            let signing_key = self.derive_signing_key(&derivation_path, password)?;
            
            // Create signature
            let signature = signing_key.sign(&transaction.tx_hash);
            let public_key = signing_key.verifying_key();
            
            let tx_signature = TransactionSignature {
                signature: signature.to_bytes().to_vec(),
                public_key: public_key.to_bytes().to_vec(),
                algorithm: SignatureAlgorithm::EdDSA, // For now, upgrade to RedPallas/RedJubjub later
                tx_hash: transaction.tx_hash.clone(),
            };
            
            signatures.push(tx_signature);
        }
        
        transaction.signatures = signatures;
        Ok(transaction)
    }
    
    
    fn get_derivation_path_for_note(&self, note: &ShieldedNote) -> NozyResult<String> {
        // For now, use a simple mapping based on note type and position
        // In a real implementation, this would be more sophisticated
        let base_path = match note.note_type {
            NoteType::Orchard => "m/44'/133'/0'/0",
            NoteType::Sapling => "m/44'/133'/0'/1",
        };
        
        Ok(format!("{}/{}", base_path, note.position.unwrap_or(0)))
    }
    
    
    pub fn derive_signing_key(&mut self, derivation_path: &str, password: &str) -> NozyResult<SigningKey> {
        // Check cache first
        if let Some(key) = self.signing_keys.get(derivation_path) {
            return Ok(key.clone());
        }
        
        // Derive from HD wallet
        let master_key = self.hd_wallet.get_master_key(password)?;
        // For now, use a simplified approach - derive from the master key directly
        let key_material = master_key.to_bytes();
        let signing_key = SigningKey::from_bytes(&key_material);
        
        // Cache the key
        self.signing_keys.insert(derivation_path.to_string(), signing_key.clone());
        
        Ok(signing_key)
    }
    
    
    pub fn verify_transaction(&self, transaction: &SignedTransaction) -> NozyResult<bool> {
        if transaction.signatures.len() != transaction.inputs.len() {
            return Ok(false);
        }
        
        for (i, signature) in transaction.signatures.iter().enumerate() {
            // Convert Vec<u8> to arrays for ed25519-dalek
            let public_key_bytes: [u8; 32] = signature.public_key.clone().try_into()
                .map_err(|_| NozyError::InvalidOperation("Invalid public key length".to_string()))?;
            let signature_bytes: [u8; 64] = signature.signature.clone().try_into()
                .map_err(|_| NozyError::InvalidOperation("Invalid signature length".to_string()))?;
            
            let public_key = VerifyingKey::from_bytes(&public_key_bytes)?;
            let sig = Signature::from_bytes(&signature_bytes);
            
            if public_key.verify(&transaction.tx_hash, &sig).is_err() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    
    pub fn estimate_fee_with_notes(
        &self,
        amount: u64,
        strategy: Option<NoteSelectionStrategy>,
    ) -> NozyResult<u64> {
        // Select notes to estimate fee
        let notes_to_spend = self.note_manager.select_notes_for_spending(
            amount,
            strategy,
        )?;
        
        // Calculate base fee
        let base_fee = 1000; // 0.00001 ZEC base fee
        
        // Add fee per input (more inputs = higher fee)
        let input_fee = notes_to_spend.len() as u64 * 500; // 0.000005 ZEC per input
        
        // Add fee per output
        let output_fee = 2 * 500; // 2 outputs (recipient + change) * 0.000005 ZEC
        
        // Add memo fee if present
        let memo_fee = 0; // Memos are free in Zcash
        
        let total_fee = base_fee + input_fee + output_fee + memo_fee;
        
        Ok(total_fee)
    }
    
    
    pub fn estimate_transaction_size(&self, transaction: &SignedTransaction) -> NozyResult<usize> {
        // Base transaction overhead
        let mut size = 100; // Version, locktime, etc.
        
        // Input sizes
        for input in &transaction.inputs {
            size += 32; // Commitment
            size += 8;  // Position
            size += input.merkle_path.len() * 32; // Merkle path
        }
        
        // Output sizes
        for output in &transaction.outputs {
            size += output.address.len();
            size += 8; // Value
            if let Some(ref memo) = output.memo {
                size += memo.len();
            }
        }
        
        // Signature sizes
        for _ in &transaction.signatures {
            size += 64; // EdDSA signature
            size += 32; // Public key
        }
        
        Ok(size)
    }
    
    
    pub fn serialize_transaction(&self, transaction: &SignedTransaction) -> NozyResult<Vec<u8>> {
        serde_json::to_vec(transaction)
            .map_err(|e| NozyError::Serialization(format!("Failed to serialize transaction: {}", e)))
    }
    
    
    pub fn mark_notes_spent(&mut self, transaction: &SignedTransaction, block_height: u32) -> NozyResult<()> {
        for input in &transaction.inputs {
            self.note_manager.mark_note_spent(&input.note.id, block_height)?;
        }
        Ok(())
    }
    
    
    pub fn get_note_manager(&self) -> &NoteManager {
        &self.note_manager
    }
    
    
    pub fn get_note_manager_mut(&mut self) -> &mut NoteManager {
        &mut self.note_manager
    }
}

impl From<ed25519_dalek::SignatureError> for NozyError {
    fn from(err: ed25519_dalek::SignatureError) -> Self {
        NozyError::InvalidOperation(format!("Signature error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_transaction_signing() {
        // Create test HD wallet
        let mut hd_wallet = HDWallet::new_from_seed(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "testnet"
        ).unwrap();
        
        let mut note_manager = NoteManager::new(&crate::config::NozyConfig::default());
        let mut signer = TransactionSigner::new(hd_wallet, note_manager);
        
        // Create test transaction
        let transaction = signer.build_transaction_with_notes(
            "test_address".to_string(),
            100000000, // 1 ZEC
            signer.estimate_fee_with_notes(100000000, None).unwrap(),
            Some(b"Test transaction".to_vec()),
            1000000,
            None,
        ).unwrap();
        
        // Sign transaction
        let signed_tx = signer.sign_transaction_with_notes(transaction, "default_password").unwrap();
        
        // Verify signatures
        assert!(signer.verify_transaction(&signed_tx).unwrap());
        
        // Test serialization
        let serialized = signer.serialize_transaction(&signed_tx).unwrap();
        assert!(!serialized.is_empty());
        
        // Test fee estimation
        let estimated_fee = signer.estimate_fee_with_notes(100000000, None).unwrap();
        assert!(estimated_fee > 0);
    }
    
    #[test]
    fn test_signature_verification() {
        let mut hd_wallet = HDWallet::new_from_seed(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "testnet"
        ).unwrap();
        
        let mut note_manager = NoteManager::new(&crate::config::NozyConfig::default());
        let mut signer = TransactionSigner::new(hd_wallet, note_manager);
        
        // Derive a signing key
        let signing_key = signer.derive_signing_key("m/44'/133'/0'/0/0", "default_password").unwrap();
        
        // Test data
        let tx_hash = b"test transaction hash";
        
        // Create test input
        let input = ShieldedInput {
            note: ShieldedNote {
                id: "test_note_id".to_string(),
                commitment: vec![1, 2, 3, 4],
                value: 100000000,
                randomness: vec![5, 6, 7, 8],
                merkle_path: Some(vec![vec![9, 10, 11, 12]]),
                position: Some(0),
                note_type: NoteType::Orchard,
            },
            merkle_path: vec![vec![9, 10, 11, 12]],
            position: 0,
        };
        
        // Sign and verify
        let signature = signing_key.sign(tx_hash);
        let is_valid = signing_key.verifying_key().verify(tx_hash, &signature).is_ok();
        
        assert!(is_valid);
    }
} 

// doing this is my calling and I love it and have fun builing and learning how to create a zcash wallet on zebrad feel like one of the first to do it. 
