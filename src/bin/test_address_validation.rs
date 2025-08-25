use nozy::{
    HDWallet, AddressManager, ZcashAddressType
};
use nozy::addresses::NetworkType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Comprehensive Zcash Address Validation Test");
    println!("==========================================");

    println!("\nTest 1: Address Generation");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let hd_wallet = HDWallet::new_from_seed(seed_phrase, "testnet")?;
    let mut address_manager = AddressManager::new(hd_wallet, NetworkType::Testnet);

    let mut orchard_addresses = Vec::new();
    let mut sapling_addresses = Vec::new();

    for i in 0..5 {
        let addr = address_manager.generate_orchard_address("")?;
        orchard_addresses.push(addr);
        
        let addr = address_manager.generate_sapling_address("")?;
        sapling_addresses.push(addr);
    }

    println!("  Generated {} Orchard addresses", orchard_addresses.len());
    println!("  Generated {} Sapling addresses", sapling_addresses.len());

    println!("\nTest 2: Address Format Validation");
    let mut orchard_valid = 0;
    let mut sapling_valid = 0;

    for addr in &orchard_addresses {
        if address_manager.validate_address(&addr.address) {
            orchard_valid += 1;
        }
    }

    for addr in &sapling_addresses {
        if address_manager.validate_address(&addr.address) {
            sapling_valid += 1;
        }
    }

    println!("  Orchard addresses valid: {}/{}", orchard_valid, orchard_addresses.len());
    println!("  Sapling addresses valid: {}/{}", sapling_valid, sapling_addresses.len());

    println!("\nTest 3: Deterministic Generation");
    let hd_wallet2 = HDWallet::new_from_seed(seed_phrase, "testnet")?;
    let mut address_manager2 = AddressManager::new(hd_wallet2, NetworkType::Testnet);
    
    let addr1 = address_manager2.generate_orchard_address("")?;
    let addr2 = address_manager2.generate_sapling_address("")?;

    let first_orchard = &orchard_addresses[0];
    let first_sapling = &sapling_addresses[0];

    let orchard_deterministic = addr1.address == first_orchard.address;
    let sapling_deterministic = addr2.address == first_sapling.address;

    println!("  Orchard addresses deterministic: {}", orchard_deterministic);
    println!("  Sapling addresses deterministic: {}", sapling_deterministic);

    println!("\nTest 4: Network Context");
    for (i, addr) in orchard_addresses.iter().enumerate() {
        println!("  Orchard {}: Network = {:?}", i + 1, addr.network);
    }

    for (i, addr) in sapling_addresses.iter().enumerate() {
        println!("  Sapling {}: Network = {:?}", i + 1, addr.network);
    }

    println!("\nTest 5: Derivation Path Consistency");
    for (i, addr) in orchard_addresses.iter().enumerate() {
        let expected_path = format!("m/44'/133'/0'/0/{}", i);
        let actual_path = &addr.derivation_path;
        let path_correct = actual_path == &expected_path;
        println!("  Orchard {}: Path = {} (Correct: {})", i + 1, actual_path, path_correct);
    }

    for (i, addr) in sapling_addresses.iter().enumerate() {
        let expected_path = format!("m/44'/133'/0'/0/{}", i);
        let actual_path = &addr.derivation_path;
        let path_correct = actual_path == &expected_path;
        println!("  Sapling {}: Path = {} (Correct: {})", i + 1, actual_path, path_correct);
    }

    println!("\nTest 6: Invalid Address Rejection");
    let long_u = "u".to_owned() + &"1".repeat(100);
    let long_z = "z".to_owned() + &"1".repeat(100);

    let invalid_addresses = vec![
        "u123",
        "z123",
        "t1invalid",
        &long_u,
        &long_z,
        "u1invalidhex",
        "z1invalidhex",
    ];

    let mut rejected_count = 0;
    for addr in &invalid_addresses {
        if !address_manager.validate_address(addr) {
            rejected_count += 1;
        }
    }

    println!("  Invalid addresses rejected: {}/{}", rejected_count, invalid_addresses.len());

    println!("\nFinal Results");
    println!("==============");
    println!("  Total addresses generated: {}", orchard_addresses.len() + sapling_addresses.len());
    println!("  Valid addresses: {}", orchard_valid + sapling_valid);
    println!("  Deterministic generation: {}", orchard_deterministic && sapling_deterministic);
    println!("  Invalid addresses rejected: {}", rejected_count);

    Ok(())
} 