use nozy::{
    HDWallet, 
    AddressType, 
    NoteManager, 
    ShieldedNote, 
    NoteType, 
    notes::NoteSelectionStrategy,
    TransactionSigner
};
use nozy::config::NozyConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing Nozy Real Note Management System...\n");
    
    // Test 1: Note Creation and Management
    test_note_creation_and_management()?;
    
    // Test 2: Note Selection Strategies
    test_note_selection_strategies()?;
    
    // Test 3: Commitment Tree Operations
    test_commitment_tree_operations()?;
    
    // Test 4: Transaction Building with Real Notes
    test_transaction_building_with_notes()?;
    
    // Test 5: Note Consolidation
    test_note_consolidation()?;
    
    println!("\n🎉 All Note Management Tests Completed!");
    Ok(())
}

fn test_note_creation_and_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test 1: Note Creation and Management");
    println!("   Creating HD wallet and note manager...");
    
    // Create HD wallet with test seed
    let mut hd_wallet = HDWallet::new_from_seed(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "testnet"
    ).unwrap();
    
    let mut note_manager = NoteManager::new(&NozyConfig::default()).unwrap();
    
    // Create some test notes
    let note1 = note_manager.create_note(
        100_000_000, // 1 ZEC
        "o1234567890abcdef".to_string(),
        Some(b"Test note 1".to_vec()),
        NoteType::Orchard,
        1_000_000, // Block height
        Some(vec![1, 2, 3, 4]), // Mock tx hash
    ).unwrap();
    
    let note2 = note_manager.create_note(
        50_000_000, // 0.5 ZEC
        "z9876543210fedcba".to_string(),
        Some(b"Test note 2".to_vec()),
        NoteType::Sapling,
        1_000_001, // Block height
        Some(vec![5, 6, 7, 8]), // Mock tx hash
    ).unwrap();
    
    // Add notes to the manager
    note_manager.add_note(note1.clone())?;
    note_manager.add_note(note2.clone())?;
    
    println!("   ✅ Created {} notes successfully", note_manager.get_commitment_tree_size());
    println!("   - Note 1: {} ZEC (Orchard)", note1.value as f64 / 100_000_000.0);
    println!("   - Note 2: {} ZEC (Sapling)", note2.value as f64 / 100_000_000.0);
    
    // Test balance calculations
    let total_balance = note_manager.get_total_balance();
    let orchard_balance = note_manager.get_balance_by_type(NoteType::Orchard);
    let sapling_balance = note_manager.get_balance_by_type(NoteType::Sapling);
    
    println!("   💰 Balance Summary:");
    println!("      - Total: {} ZEC", total_balance as f64 / 100_000_000.0);
    println!("      - Orchard: {} ZEC", orchard_balance as f64 / 100_000_000.0);
    println!("      - Sapling: {} ZEC", sapling_balance as f64 / 100_000_000.0);
    
    // Test commitment tree
    let tree_root = note_manager.get_commitment_tree_root();
    println!("   🌳 Commitment Tree:");
    println!("      - Size: {}", note_manager.get_commitment_tree_size());
    println!("      - Root: {}...", hex::encode(&tree_root[..8]));
    
    println!();
    Ok(())
}

fn test_note_selection_strategies() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test 2: Note Selection Strategies");
    println!("   Testing different note selection strategies...");
    
    let mut note_manager = NoteManager::new(&NozyConfig::default()).unwrap();
    
    // Create notes with different values and types
    let notes_data = vec![
        (100_000_000, NoteType::Orchard, 1_000_000),    // 1 ZEC, Orchard, older
        (75_000_000, NoteType::Sapling, 1_000_001),     // 0.75 ZEC, Sapling, newer
        (200_000_000, NoteType::Orchard, 1_000_002),    // 2 ZEC, Orchard, newest
        (25_000_000, NoteType::Sapling, 1_000_003),     // 0.25 ZEC, Sapling, newest
    ];
    
    for (value, note_type, height) in notes_data {
        let note = note_manager.create_note(
            value,
            format!("test_address_{}", value),
            None,
            note_type,
            height,
            None,
        ).unwrap();
        
        // Add note to manager
        note_manager.add_note(note)?;
    }
    
    let amount_needed = 150_000_000; // 1.5 ZEC
    
    // Test PrivacyFirst strategy
    let privacy_notes = note_manager.select_notes_for_spending(
        amount_needed,
        Some(NoteSelectionStrategy::PrivacyFirst)
    ).unwrap();
    
    println!("   🔒 PrivacyFirst Strategy:");
    println!("      - Selected {} notes", privacy_notes.len());
    println!("      - Total value: {} ZEC", 
        privacy_notes.iter().map(|n| n.value).sum::<u64>() as f64 / 100_000_000.0);
    println!("      - Types: {:?}", 
        privacy_notes.iter().map(|n| &n.note_type).collect::<Vec<_>>());
    
    // Test EfficiencyFirst strategy
    let efficiency_notes = note_manager.select_notes_for_spending(
        amount_needed,
        Some(NoteSelectionStrategy::EfficiencyFirst)
    ).unwrap();
    
    println!("   ⚡ EfficiencyFirst Strategy:");
    println!("      - Selected {} notes", efficiency_notes.len());
    println!("      - Total value: {} ZEC", 
        efficiency_notes.iter().map(|n| n.value).sum::<u64>() as f64 / 100_000_000.0);
    println!("      - Types: {:?}", 
        efficiency_notes.iter().map(|n| &n.note_type).collect::<Vec<_>>());
    
    // Test ValueBased strategy
    let value_notes = note_manager.select_notes_for_spending(
        amount_needed,
        Some(NoteSelectionStrategy::ValueBased)
    ).unwrap();
    
    println!("   💎 ValueBased Strategy:");
    println!("      - Selected {} notes", value_notes.len());
    println!("      - Total value: {} ZEC", 
        value_notes.iter().map(|n| n.value).sum::<u64>() as f64 / 100_000_000.0);
    println!("      - Values: {:?}", 
        value_notes.iter().map(|n| n.value as f64 / 100_000_000.0).collect::<Vec<_>>());
    
    println!();
    Ok(())
}

fn test_commitment_tree_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test 3: Commitment Tree Operations");
    println!("   Testing commitment tree functionality...");
    
    let mut note_manager = NoteManager::new(&NozyConfig::default()).unwrap();
    
    // Create multiple notes to build the tree
    for i in 0..5 {
        let note = note_manager.create_note(
            10_000_000 * (i + 1) as u64, // 0.1, 0.2, 0.3, 0.4, 0.5 ZEC
            format!("test_address_{}", i),
            Some(format!("Test note {}", i).into_bytes()),
            if i % 2 == 0 { NoteType::Orchard } else { NoteType::Sapling },
            1_000_000 + i as u32,
            Some(vec![i as u8; 8]),
        ).unwrap();
        
        // Add note to manager
        note_manager.add_note(note)?;
    }
    
    println!("   🌳 Tree Statistics:");
    println!("      - Total notes: {}", note_manager.get_commitment_tree_size());
    println!("      - Tree depth: 32 (fixed)");
    println!("      - Root hash: {}...", 
        hex::encode(&note_manager.get_commitment_tree_root()[..8]));
    
    // Test merkle path calculation
    let all_notes: Vec<&ShieldedNote> = note_manager.get_unspent_notes();
    if let Some(first_note) = all_notes.first() {
        if let Some(merkle_path) = &first_note.merkle_path {
            println!("   📍 Merkle Path for First Note:");
            println!("      - Position: {}", first_note.position.unwrap_or(0));
            println!("      - Path length: {}", merkle_path.len());
            println!("      - Path hashes: {}...", 
                hex::encode(&merkle_path[0][..8]));
        }
    }
    
    println!();
    Ok(())
}

fn test_transaction_building_with_notes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test 4: Transaction Building with Real Notes");
    println!("   Testing transaction creation using real notes...");
    
    let mut hd_wallet = HDWallet::new_from_seed(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "testnet"
    ).unwrap();
    
    let mut note_manager = NoteManager::new(&NozyConfig::default()).unwrap();
    
    // Create notes for testing
    let note1 = note_manager.create_note(
        100_000_000, // 1 ZEC
        "o1234567890abcdef".to_string(),
        None,
        NoteType::Orchard,
        1_000_000,
        None,
    ).unwrap();
    
    let note2 = note_manager.create_note(
        50_000_000, // 0.5 ZEC
        "o1234567890abcdef".to_string(),
        None,
        NoteType::Orchard,
        1_000_001,
        None,
    ).unwrap();
    
    // Add notes to manager
    note_manager.add_note(note1)?;
    note_manager.add_note(note2)?;
    
    let mut signer = TransactionSigner::new(hd_wallet, note_manager);
    
    // Build transaction
    let transaction = signer.build_transaction_with_notes(
        "o9876543210fedcba".to_string(), // Recipient
        75_000_000, // 0.75 ZEC
        10_000, // 0.0001 ZEC fee
        Some(b"Test transaction with real notes".to_vec()),
        1_000_000, // Expiry height
        Some(NoteSelectionStrategy::PrivacyFirst),
    ).unwrap();
    
    println!("   📤 Transaction Built Successfully:");
    println!("      - Inputs: {} notes", transaction.inputs.len());
    println!("      - Outputs: {} (including change)", transaction.outputs.len());
    println!("      - Fee: {} ZEC", transaction.fee as f64 / 100_000_000.0);
    println!("      - Change: {} ZEC", 
        transaction.change_output.as_ref().map(|c| c.value as f64 / 100_000_000.0).unwrap_or(0.0));
    
    // Test fee estimation
    let estimated_fee = signer.estimate_fee_with_notes(
        75_000_000,
        Some(NoteSelectionStrategy::PrivacyFirst)
    ).unwrap();
    
    println!("   💰 Fee Estimation:");
    println!("      - Estimated fee: {} ZEC", estimated_fee as f64 / 100_000_000.0);
    println!("      - Actual fee: {} ZEC", transaction.fee as f64 / 100_000_000.0);
    
    println!();
    Ok(())
}

fn test_note_consolidation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test 5: Note Consolidation");
    println!("   Testing note consolidation for efficiency...");
    
    let mut note_manager = NoteManager::new(&NozyConfig::default()).unwrap();
    
    // Create many small notes
    for i in 0..15 {
        let note = note_manager.create_note(
            5_000_000, // 0.05 ZEC each (small notes)
            format!("test_address_{}", i),
            None,
            NoteType::Orchard,
            1_000_000 + i as u32,
            None,
        ).unwrap();
        
        // Add note to manager
        note_manager.add_note(note)?;
    }
    
    println!("   📊 Before Consolidation:");
    println!("      - Total notes: {}", note_manager.get_commitment_tree_size());
    println!("      - Total balance: {} ZEC", 
        note_manager.get_total_balance() as f64 / 100_000_000.0);
    
    // Consolidate notes
    let consolidated_notes = note_manager.consolidate_notes().unwrap();
    
    println!("   🔄 After Consolidation:");
    println!("      - Consolidated {} notes", consolidated_notes.len());
    println!("      - Remaining notes: {}", note_manager.get_commitment_tree_size());
    println!("      - Total balance: {} ZEC", 
        note_manager.get_total_balance() as f64 / 100_000_000.0);
    
    if let Some(consolidated) = consolidated_notes.first() {
        println!("   💎 Consolidated Note:");
        println!("      - Value: {} ZEC", consolidated.value as f64 / 100_000_000.0);
        println!("      - Type: {:?}", consolidated.note_type);
        println!("      - Commitment: {}...", 
            hex::encode(&consolidated.commitment[..8]));
    }
    
    println!();
    Ok(())
} 