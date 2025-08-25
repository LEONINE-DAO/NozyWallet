use nozy::{
    HDWallet, 
    TransactionSigner, 
    NoteManager,
    NoteType,
    NozyConfig
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Testing Nozy Transaction Signing System...\n");
    
   
    test_basic_transaction_signing()?;
    
    
    test_fee_estimation()?;
    
    
    test_note_selection()?;
    
    println!("\n🎉 All Transaction Signing Tests Completed!");
    Ok(())
}

fn test_basic_transaction_signing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test 1: Basic Transaction Signing with Notes");
    println!("   Creating HD wallet and signing transaction with real notes...");
    
    let hd_wallet = HDWallet::new_from_seed(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "testnet"
    )?;
    
    let config = NozyConfig::default();
    let mut note_manager = NoteManager::new(&config)?;
    
    let test_note = note_manager.create_note(
        100_000_000, // 1 ZEC
        "test_recipient".to_string(),
        Some("Test memo".as_bytes().to_vec()),
        NoteType::Orchard,
        500000, // block height
        None // tx_hash
    )?;
    
    note_manager.add_note(test_note)?;
    
    let mut signer = TransactionSigner::new(hd_wallet, note_manager);
    
    let recipient = "u1test_recipient_address_for_testing_purposes_only_12345678".to_string();
    let amount = 50_000_000; // 0.5 ZEC
    
    println!("   Building transaction...");
    let signed_transaction = signer.build_transaction_with_notes(
        recipient,
        amount,
        10_000, // 0.0001 ZEC fee
        Some("Test transaction memo".as_bytes().to_vec()),
        1_000_000, // expiry height
        None // Use default note selection strategy
    )?;
    
    println!("   ✅ Transaction built successfully!");
    println!("      Transaction Hash: {}", hex::encode(&signed_transaction.tx_hash));
    println!("      Inputs: {}", signed_transaction.inputs.len());
    println!("      Outputs: {}", signed_transaction.outputs.len());
    println!("      Fee: {} zatoshi", signed_transaction.fee);
    
    Ok(())
}

fn test_fee_estimation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 Test 2: Fee Estimation");
    println!("   Testing fee estimation for different transaction sizes...");
    
    let hd_wallet = HDWallet::new_from_seed(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "testnet"
    )?;
    
    let config = NozyConfig::default();
    let note_manager = NoteManager::new(&config)?;
    let signer = TransactionSigner::new(hd_wallet, note_manager);
    
    let test_amounts = vec![10_000_000, 50_000_000, 100_000_000]; // 0.1, 0.5, 1 ZEC
    
    for amount in test_amounts {
        println!("   Testing fee estimation for {} zatoshi...", amount);
        
        let estimated_fee = signer.estimate_fee_with_notes(
            amount,
            None // Use default strategy
        )?;
        
        println!("      ✅ Estimated fee: {} zatoshi", estimated_fee);
    }
    
    Ok(())
}

fn test_note_selection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧪 Test 3: Note Selection Strategies");
    println!("   Testing different note selection strategies...");
    
    let hd_wallet = HDWallet::new_from_seed(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "testnet"
    )?;
    
    let config = NozyConfig::default();
    let mut note_manager = NoteManager::new(&config)?;
    
    let note_values = vec![25_000_000, 50_000_000, 75_000_000, 100_000_000]; // 0.25, 0.5, 0.75, 1 ZEC
    
    for (i, value) in note_values.iter().enumerate() {
        let test_note = note_manager.create_note(
            *value,
            format!("test_recipient_{}", i),
            Some(format!("Test memo {}", i).as_bytes().to_vec()),
            if i % 2 == 0 { NoteType::Orchard } else { NoteType::Sapling },
            500000 + i as u32, // block height
            None // tx_hash
        )?;
        
        note_manager.add_note(test_note)?;
    }
    
    println!("   Created {} test notes", note_values.len());
    println!("   Total balance: {} zatoshi", note_manager.get_total_balance());
    
    let mut signer = TransactionSigner::new(hd_wallet, note_manager);
    
    let recipient = "u1test_recipient_address_for_note_selection_testing_12345678".to_string();
    let amount = 60_000_000; // 0.6 ZEC (should select the 75M note or combine smaller ones)
    
    println!("   Building transaction requiring note selection...");
    let signed_transaction = signer.build_transaction_with_notes(
        recipient,
        amount,
        10_000, // 0.0001 ZEC fee
        Some("Note selection test".as_bytes().to_vec()),
        1_000_000, // expiry height
        None // Use default note selection strategy
    )?;
    
    println!("   ✅ Transaction built with note selection!");
    println!("      Selected {} input notes", signed_transaction.inputs.len());
    println!("      Total input value covers {} zatoshi transaction", amount);
    
    Ok(())
} 