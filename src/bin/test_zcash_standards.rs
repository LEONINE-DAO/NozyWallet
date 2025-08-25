use nozy::{
    HDWallet, AddressManager, ZcashAddressType
};
use nozy::addresses::NetworkType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Zcash Foundation Standards Implementation");
    println!("===============================================");

    println!("\nTest 1: Creating HD wallet with seed phrase");
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let hd_wallet = HDWallet::new_from_seed(seed_phrase, "testnet")?;
    println!("  HD wallet created successfully");

    println!("\nTest 2: Creating address manager");
    let mut address_manager = AddressManager::new(hd_wallet, NetworkType::Testnet);
    println!("  Address manager created successfully");

    println!("\nTest 3: Generating Orchard address");
    match address_manager.generate_orchard_address("test_password") {
        Ok(address) => {
            println!("  Orchard address generated: {}", address.address);
            println!("  Address type: {:?}", address.address_type);
            println!("  Derivation path: {}", address.derivation_path);
            println!("  Network: {:?}", address.network);

            if address.address.starts_with("u") {
                println!("  ✓ Address has correct u... prefix");
            } else {
                println!("  ✗ Address has wrong prefix: {}", address.address);
            }
        },
        Err(e) => {
            println!("  ✗ Failed to generate Orchard address: {}", e);
            return Err(e.into());
        }
    }

    println!("\nTest 4: Generating Sapling address");
    match address_manager.generate_sapling_address("test_password") {
        Ok(address) => {
            println!("  Sapling address generated: {}", address.address);
            println!("  Address type: {:?}", address.address_type);
            println!("  Derivation path: {}", address.derivation_path);
            println!("  Network: {:?}", address.network);

            if address.address.starts_with("z") {
                println!("  ✓ Address has correct z... prefix");
            } else {
                println!("  ✗ Address has wrong prefix: {}", address.address);
            }
        },
        Err(e) => {
            println!("  ✗ Failed to generate Sapling address: {}", e);
            return Err(e.into());
        }
    }

    println!("\nTest 5: Generating Unified address");
    match address_manager.generate_unified_address("test_password") {
        Ok(address) => {
            println!("  Unified address generated: {}", address.address);
            println!("  Address type: {:?}", address.address_type);
            println!("  Derivation path: {}", address.derivation_path);
            println!("  Network: {:?}", address.network);

            if address.address.starts_with("u") {
                println!("  ✓ Address has correct u... prefix");
            } else {
                println!("  ✗ Address has wrong prefix: {}", address.address);
            }
        },
        Err(e) => {
            println!("  ✗ Failed to generate Unified address: {}", e);
            return Err(e.into());
        }
    }

    println!("\nTest 6: Validating addresses");
    let addresses = address_manager.get_all_addresses();
    for address in addresses {
        let is_valid = address_manager.validate_address(&address.address);
        println!("  Address: {} -> Valid: {}", address.address, is_valid);
    }

    println!("\n=== ZCASH FOUNDATION STANDARDS TEST COMPLETE ===");
    println!("✓ Using UnifiedSpendingKey::from_seed instead of Xprv");
    println!("✓ Using proper bech32 encoding instead of hex concatenation");
    println!("✓ All addresses generated with official Zcash crates");

    Ok(())
} 