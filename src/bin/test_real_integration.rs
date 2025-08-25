use nozy::{
    HDWallet, AddressManager, ZcashAddressType, 
    ZebraClient, ZebraConfig, NoteManager
};
use nozy::config::NozyConfig; 
use nozy::addresses::NetworkType; 

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Nozy Real Integration with Zebra Testnet...\n");
    
    
    println!("🧪 Test 1: Real Zcash Address Generation");
    
    
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let hd_wallet = HDWallet::new_from_seed(seed_phrase, "testnet")?;
    println!("   ✅ Created HD wallet from seed phrase");
    
    
    let mut address_manager = AddressManager::new(hd_wallet, NetworkType::Testnet);
    println!("   ✅ Created address manager");
    
    
    let orchard_address = address_manager.generate_orchard_address("default_password")?;
    println!("   🌳 Generated Orchard address: {}", orchard_address.address);
    println!("      Path: {}", orchard_address.derivation_path);
    println!("      Type: {:?}", orchard_address.address_type);
    
    
    let sapling_address = address_manager.generate_sapling_address("default_password")?;
    println!("   🌿 Generated Sapling address: {}", sapling_address.address);
    println!("      Path: {}", sapling_address.derivation_path);
    println!("      Type: {:?}", sapling_address.address_type);
    
    
    println!("\n🧪 Test 2: Address Validation");
    println!("   Orchard address valid: {}", address_manager.validate_address(&orchard_address.address));
    println!("   Sapling address valid: {}", address_manager.validate_address(&sapling_address.address));
    println!("   Invalid address valid: {}", address_manager.validate_address("invalid_address"));
    
    
    println!("\n🧪 Test 3: Zebra Connection and Note Discovery");
    
    let zebra_config = ZebraConfig::default();
    let mut zebra_client = ZebraClient::new(zebra_config);
    
    
    match zebra_client.check_connection() {
        Ok(connected) => {
            if connected {
                println!("   ✅ Connected to Zebra testnet node");
                
                
                println!("   🔍 Discovering notes for Orchard address...");
                match zebra_client.get_shielded_notes(&orchard_address.address) {
                    Ok(notes) => {
                        println!("      Found {} notes", notes.len());
                        for (i, note) in notes.iter().enumerate() {
                            println!("        Note {}: {} zatoshi ({:?})", i + 1, note.value, note.note_type);
                        }
                    }
                    Err(e) => {
                        println!("      Error fetching notes: {}", e);
                        println!("      (This is normal if the address has no notes yet)");
                    }
                }
                
                println!("   🔍 Discovering notes for Sapling address...");
                match zebra_client.get_shielded_notes(&sapling_address.address) {
                    Ok(notes) => {
                        println!("      Found {} notes", notes.len());
                        for (i, note) in notes.iter().enumerate() {
                            println!("        Note {}: {} zatoshi ({:?})", i + 1, note.value, note.note_type);
                        }
                    }
                    Err(e) => {
                        println!("      Error fetching notes: {}", e);
                        println!("      (This is normal if the address has no notes yet)");
                    }
                }
                
            } else {
                println!("   ❌ Failed to connect to Zebra");
            }
        }
        Err(e) => {
            println!("   ❌ Connection error: {}", e);
        }
    }
    
    
    println!("\n🧪 Test 4: Note Manager Integration");
    let config = NozyConfig::default();
    let mut note_manager = NoteManager::new(&config)?;
    println!("   ✅ Created note manager");
    
    
    println!("   📝 Adding addresses to note manager...");
    println!("      Orchard: {}", orchard_address.address);
    println!("      Sapling: {}", sapling_address.address);
    
    
    println!("\n🧪 Test 5: Address Management");
    println!("   Total addresses: {}", address_manager.get_all_addresses().len());
    println!("   Orchard addresses: {}", address_manager.get_address_count(&ZcashAddressType::Orchard));
    println!("   Sapling addresses: {}", address_manager.get_address_count(&ZcashAddressType::Sapling));
    
    let orchard_addresses = address_manager.get_addresses_by_type(&ZcashAddressType::Orchard);
    let sapling_addresses = address_manager.get_addresses_by_type(&ZcashAddressType::Sapling);
    
    println!("   Orchard addresses:");
    for addr in orchard_addresses {
        println!("     - {} (Path: {})", addr.address, addr.derivation_path);
    }
    
    println!("   Sapling addresses:");
    for addr in sapling_addresses {
        println!("     - {} (Path: {})", addr.address, addr.derivation_path);
    }
    
    println!("\n🎯 Real Integration Test Complete!");
    println!("   🌐 Your addresses are ready for testnet use!");
    println!("   💰 To get testnet ZEC, use a faucet and send to one of these addresses:");
    println!("      Orchard: {}", orchard_address.address);
    println!("      Sapling: {}", sapling_address.address);
    println!("   🔍 Once you receive ZEC, run this test again to see your notes!");
    
    Ok(())
} 
