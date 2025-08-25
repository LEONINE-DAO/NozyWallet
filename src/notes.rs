//! Note management for Orchard and Sapling notes no T address here

use crate::error::NozyResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use blake2b_simd::Params;
use sha2::{Sha256, Digest};
use rand::RngCore;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedNote {
    
    pub id: String,
    
    
    pub note_type: NoteType,
    
    
    pub value: u64,
    
    
    pub commitment: Vec<u8>,
    
    
    pub nullifier: Option<Vec<u8>>,
    
    
    pub recipient_address: String,
    
    
    pub memo: Option<Vec<u8>>,
    
    
    pub randomness: Vec<u8>,
    
    
    pub created_at_height: u32,
    
    
    pub spent_at_height: Option<u32>,
    
    
    pub tx_hash: Option<Vec<u8>>,
    
    
    pub merkle_path: Option<Vec<Vec<u8>>>,
    
    
    pub position: Option<u64>,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NoteType {
    Orchard,
    Sapling,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoteSelectionStrategy {
    
    PrivacyFirst,
    
    
    EfficiencyFirst,
    
    
    Balanced,
    
    
    ValueBased,
    
    
    AgeBased,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteManagerConfig {
    
    pub default_strategy: NoteSelectionStrategy,
    
    
    pub enable_consolidation: bool,
    
    
    pub min_consolidation_value: u64,
    
    
    pub max_consolidation_notes: usize,
    
    
    pub enable_note_mixing: bool,
    
    
    pub mixing_rounds: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteManager {
    
    notes: HashMap<String, ShieldedNote>,
    
    
    config: NoteManagerConfig,
    
    
    commitment_tree: CommitmentTree,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentTree {
    
    pub depth: u32,
    
    
    pub size: u64,
    
    
    pub root: Vec<u8>,
    
    
    pub nodes: Vec<Vec<u8>>,
}

impl NoteManager {
    
    pub fn new(config: &crate::config::NozyConfig) -> NozyResult<Self> {
        Ok(Self {
            notes: HashMap::new(),
            config: NoteManagerConfig {
                default_strategy: NoteSelectionStrategy::PrivacyFirst,
                enable_consolidation: config.privacy.enable_orchard,
                min_consolidation_value: 10_000, // 0.0001 ZEC
                max_consolidation_notes: 10,
                enable_note_mixing: true,
                mixing_rounds: 3,
            },
            commitment_tree: CommitmentTree {
                depth: 32,
                size: 0,
                root: vec![0u8; 32],
                nodes: Vec::new(),
            },
        })
    }
    
    
    pub fn create_note(
        &mut self,
        value: u64,
        recipient_address: String,
        memo: Option<Vec<u8>>,
        note_type: NoteType,
        block_height: u32,
        tx_hash: Option<Vec<u8>>,
    ) -> NozyResult<ShieldedNote> {
        let mut rng = rand::thread_rng();
        
        // Generate randomness for note commitment
        let randomness = {
            let mut bytes = vec![0u8; 32];
            rng.fill_bytes(&mut bytes);
            bytes
        };
        
        // Calculate real note commitment
        let commitment = self.calculate_note_commitment(
            value,
            &recipient_address,
            &randomness,
            &note_type,
        )?;
        
        // Generate unique note ID
        let note_id = self.generate_note_id(&commitment, &recipient_address);
        
        // Create the note
        let note = ShieldedNote {
            id: note_id,
            note_type,
            value,
            commitment,
            nullifier: None,
            recipient_address,
            memo,
            randomness,
            created_at_height: block_height,
            spent_at_height: None,
            tx_hash,
            merkle_path: None,
            position: None,
        };
        
        // Add to commitment tree
        self.add_note_to_tree(&note)?;
        
        Ok(note)
    }
    
    
    fn calculate_note_commitment(
        &self,
        value: u64,
        recipient_address: &str,
        randomness: &[u8],
        note_type: &NoteType,
    ) -> NozyResult<Vec<u8>> {
        let mut hasher = Params::new()
            .hash_length(32)
            .to_state();
        
        // Hash note components
        hasher.update(&value.to_le_bytes());
        hasher.update(recipient_address.as_bytes());
        hasher.update(randomness);
        
        // Add note type identifier
        let type_bytes = match note_type {
            NoteType::Orchard => b"orchard",
            NoteType::Sapling => b"sapling",
        };
        hasher.update(type_bytes);
        
        Ok(hasher.finalize().as_bytes().to_vec())
    }
    
    
    fn generate_note_id(&self, commitment: &[u8], address: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(commitment);
        hasher.update(address.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..8]) // First 8 bytes as hex
    }
    
    
    fn add_note_to_tree(&mut self, note: &ShieldedNote) -> NozyResult<()> {
        // Add commitment to tree
        self.commitment_tree.nodes.push(note.commitment.clone());
        self.commitment_tree.size += 1;
        
        // Recalculate root hash
        self.commitment_tree.root = self.calculate_tree_root()?;
        
        // Update note position
        let position = self.commitment_tree.size - 1;
        
        // Calculate merkle path
        let merkle_path = self.calculate_merkle_path(position)?;
        
        // Update note with position and merkle path
        if let Some(existing_note) = self.notes.get_mut(&note.id) {
            existing_note.position = Some(position);
            existing_note.merkle_path = Some(merkle_path);
        }
        
        Ok(())
    }
    
    
    fn calculate_tree_root(&self) -> NozyResult<Vec<u8>> {
        if self.commitment_tree.nodes.is_empty() {
            return Ok(vec![0u8; 32]);
        }
        
        let mut current_level = self.commitment_tree.nodes.clone();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let mut hasher = Params::new()
                    .hash_length(32)
                    .to_state();
                
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // Duplicate for odd number
                }
                
                next_level.push(hasher.finalize().as_bytes().to_vec());
            }
            
            current_level = next_level;
        }
        
        Ok(current_level[0].clone())
    }
    
    
    fn calculate_merkle_path(&self, position: u64) -> NozyResult<Vec<Vec<u8>>> {
        let mut path = Vec::new();
        let mut current_pos = position;
        let mut current_level_size = self.commitment_tree.size;
        
        while current_level_size > 1 {
            let sibling_pos = if current_pos % 2 == 0 {
                current_pos + 1
            } else {
                current_pos - 1
            };
            
            if sibling_pos < current_level_size {
                path.push(self.commitment_tree.nodes[sibling_pos as usize].clone());
            } else {
                // Sibling doesn't exist, use current node
                path.push(self.commitment_tree.nodes[current_pos as usize].clone());
            }
            
            current_pos /= 2;
            current_level_size = (current_level_size + 1) / 2;
        }
        
        Ok(path)
    }
    
    
    pub fn add_note(&mut self, note: ShieldedNote) -> NozyResult<()> {
        let note_id = note.id.clone();
        self.notes.insert(note_id.clone(), note);
        Ok(())
    }
    
    
    pub fn get_note(&self, id: &str) -> Option<&ShieldedNote> {
        self.notes.get(id)
    }
    
    
    pub fn get_unspent_notes(&self) -> Vec<&ShieldedNote> {
        self.notes.values()
            .filter(|note| note.nullifier.is_none())
            .collect()
    }
    
    
    pub fn get_unspent_notes_by_type(&self, note_type: NoteType) -> Vec<&ShieldedNote> {
        self.notes.values()
            .filter(|note| note.note_type == note_type && note.nullifier.is_none())
            .collect()
    }
    
    
    pub fn get_total_balance(&self) -> u64 {
        self.get_unspent_notes()
            .iter()
            .map(|note| note.value)
            .sum()
    }
    
    
    pub fn get_balance_by_type(&self, note_type: NoteType) -> u64 {
        self.get_unspent_notes_by_type(note_type)
            .iter()
            .map(|note| note.value)
            .sum()
    }
    
    
    pub fn select_notes_for_spending(
        &self,
        amount: u64,
        strategy: Option<NoteSelectionStrategy>,
    ) -> NozyResult<Vec<&ShieldedNote>> {
        let strategy = strategy.unwrap_or(self.config.default_strategy.clone());
        let mut unspent_notes = self.get_unspent_notes();
        
        match strategy {
            NoteSelectionStrategy::PrivacyFirst => {
                // Prefer Orchard notes first
                unspent_notes.sort_by(|a, b| {
                    match (a.note_type, b.note_type) {
                        (NoteType::Orchard, NoteType::Sapling) => std::cmp::Ordering::Less,
                        (NoteType::Sapling, NoteType::Orchard) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            }
            NoteSelectionStrategy::EfficiencyFirst => {
                // Prefer Sapling notes first (smaller proofs)
                unspent_notes.sort_by(|a, b| {
                    match (a.note_type, b.note_type) {
                        (NoteType::Sapling, NoteType::Orchard) => std::cmp::Ordering::Less,
                        (NoteType::Orchard, NoteType::Sapling) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
            }
            NoteSelectionStrategy::ValueBased => {
                // Prefer larger notes
                unspent_notes.sort_by(|a, b| b.value.cmp(&a.value));
            }
            NoteSelectionStrategy::AgeBased => {
                // Prefer older notes
                unspent_notes.sort_by(|a, b| a.created_at_height.cmp(&b.created_at_height));
            }
            NoteSelectionStrategy::Balanced => {
                // Mix of both types
                // Keep current order
            }
        }
        
        // Select notes to cover the amount
        let mut selected_notes = Vec::new();
        let mut total_selected = 0u64;
        
        for note in unspent_notes {
            if total_selected >= amount {
                break;
            }
            selected_notes.push(note);
            total_selected += note.value;
        }
        
        if total_selected < amount {
            return Err(crate::error::NozyError::InsufficientFunds(
                format!("Insufficient funds. Required: {}, Available: {}", amount, total_selected)
            ));
        }
        
        Ok(selected_notes)
    }
    
    
    pub fn mark_note_spent(&mut self, note_id: &str, spent_height: u32) -> NozyResult<()> {
        // Get the note data first to avoid borrowing conflicts
        let note_data = if let Some(note) = self.notes.get(note_id) {
            note.clone()
        } else {
            return Ok(());
        };
        
        // Generate nullifier from the cloned data
        let nullifier = self.generate_note_nullifier(&note_data)?;
        
        // Now update the note
        if let Some(note) = self.notes.get_mut(note_id) {
            note.spent_at_height = Some(spent_height);
            note.nullifier = Some(nullifier);
        }
        
        Ok(())
    }
    
    
    fn generate_note_nullifier(&self, note: &ShieldedNote) -> NozyResult<Vec<u8>> {
        let mut hasher = Params::new()
            .hash_length(32)
            .to_state();
        
        hasher.update(&note.commitment);
        hasher.update(&note.randomness);
        
        // Add note type to nullifier
        let type_bytes = match note.note_type {
            NoteType::Orchard => b"orchard_nullifier",
            NoteType::Sapling => b"sapling_nullifier",
        };
        hasher.update(type_bytes);
        
        Ok(hasher.finalize().as_bytes().to_vec())
    }
    
    
    pub fn get_commitment_tree_root(&self) -> Vec<u8> {
        self.commitment_tree.root.clone()
    }
    
    
    pub fn get_commitment_tree_size(&self) -> u64 {
        self.commitment_tree.size
    }
    
    
    pub fn consolidate_notes(&mut self) -> NozyResult<Vec<ShieldedNote>> {
        if !self.config.enable_consolidation {
            return Ok(Vec::new());
        }
        
        // Collect note IDs first to avoid borrowing issues
        let small_note_ids: Vec<String> = self.notes.values()
            .filter(|note| {
                note.nullifier.is_none() && 
                note.value < self.config.min_consolidation_value
            })
            .map(|note| note.id.clone())
            .collect();
        
        if small_note_ids.len() < 2 {
            return Ok(Vec::new());
        }
        
        // Get the notes by ID to avoid borrowing conflicts
        let mut small_notes: Vec<&ShieldedNote> = small_note_ids.iter()
            .filter_map(|id| self.notes.get(id))
            .collect();
        
        // Sort by value and take the smallest notes
        small_notes.sort_by(|a, b| a.value.cmp(&b.value));
        let notes_to_consolidate: Vec<&ShieldedNote> = small_notes.clone()
            .into_iter()
            .take(self.config.max_consolidation_notes.min(small_notes.len()))
            .collect();
        
        let total_value: u64 = notes_to_consolidate.iter().map(|note| note.value).sum();
        let recipient_address = notes_to_consolidate[0].recipient_address.clone();
        
        // Create consolidated note
        let consolidated_note = self.create_note(
            total_value,
            recipient_address,
            None, // No memo for consolidated notes
            NoteType::Orchard, // Prefer Orchard for consolidation
            0, // Will be updated when actually created
            None,
        )?;
        
        // Mark original notes as spent
        for note_id in small_note_ids {
            self.mark_note_spent(&note_id, 0)?;
        }
        
        Ok(vec![consolidated_note])
    }
}

 
