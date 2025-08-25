use nozy::{HDWallet, AddressType};

fn main() {
    println!("🔑 Testing Real Child Key Derivation...\n");
    
    // Create a new HD wallet with seed phrase
    let mut hd_wallet = HDWallet::new_from_seed("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", "mainnet").unwrap();
    
    println!("✅ HD Wallet created successfully!");
    println!("   Seed phrase: {}", hd_wallet.seed_phrase.as_ref().unwrap());
    println!("   Master key: encrypted ({} bytes)", hd_wallet.encrypted_master_key.as_ref().unwrap().encrypted_data.len());
    println!();
    
    // Test different derivation paths
    let test_paths = vec![
        ("m/44'/133'/0'/0/0", AddressType::Orchard),
        ("m/44'/133'/0'/0/1", AddressType::Orchard),
        ("m/44'/133'/0'/0/0", AddressType::Sapling),
        ("m/44'/133'/0'/0/1", AddressType::Sapling),
    ];
    
    for (path, addr_type) in test_paths {
        println!("🔐 Deriving {} at path: {}", format!("{:?}", addr_type), path);
        
        match hd_wallet.derive_address(path, addr_type) {
            Ok(derived) => {
                println!("   ✅ Success! Address: {}", derived.address);
                println!("   📍 Path: {}", derived.path);
                println!("   🏷️  Type: {:?}", derived.address_type);
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
            }
        }
        println!();
    }
    
    // Test deterministic behavior (same path should give same address)
    println!("🔄 Testing Deterministic Behavior...");
    let first_derivation = hd_wallet.derive_address("m/44'/133'/0'/0/0", AddressType::Orchard).unwrap();
    let second_derivation = hd_wallet.derive_address("m/44'/133'/0'/0/0", AddressType::Orchard).unwrap();
    
    if first_derivation.address == second_derivation.address {
        println!("   ✅ Deterministic! Same path = same address");
        println!("   📍 Address: {}", first_derivation.address);
    } else {
        println!("   ❌ Non-deterministic! This is a problem!");
    }
    
    println!("\n🎉 Child Key Derivation Test Complete!");
} 