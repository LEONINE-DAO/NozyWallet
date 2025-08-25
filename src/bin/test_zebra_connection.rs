use nozy::zebra_integration::{ZebraClient, ZebraConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing Nozy Wallet Connection to Zebra Testnet...\n");
    
    
    println!("🧪 Test 1: Basic Zebra Connection");
    let config = ZebraConfig::default();
    println!("   📡 Connecting to: {}", config.rpc_endpoint);
    println!("   🌐 Network: {}", config.network);
    
    let mut client = ZebraClient::new(config);
    
    match client.check_connection() {
        Ok(connected) => {
            if connected {
                println!("   ✅ Successfully connected to Zebra testnet!");
            } else {
                println!("   ❌ Failed to connect to Zebra");
            }
        }
        Err(e) => {
            println!("   ❌ Connection error: {}", e);
        }
    }
    
    
    println!("\n🧪 Test 2: Zebra Status");
    match client.get_status() {
        Ok(status) => {
            println!("   📊 Connection: {}", status.connected);
            println!("   📦 Block Height: {:?}", status.block_height);
            println!("   🔄 Sync Status: {:?}", status.sync_status);
            println!("   🌐 Network: {}", status.network);
        }
        Err(e) => {
            println!("   ❌ Status error: {}", e);
        }
    }
    
    
    println!("\n🧪 Test 3: Network Status");
    match client.get_network_status() {
        Ok(status) => println!("   📡 {}", status),
        Err(e) => println!("   ❌ Network status error: {}", e),
    }
    
    
    println!("\n🧪 Test 4: Mempool Status");
    match client.get_mempool_info() {
        Ok(info) => println!("   📋 {}", info),
        Err(e) => println!("   ❌ Mempool error: {}", e),
    }
    
    println!("\n🎯 Connection Test Complete!");
    println!("   If you see connection errors, make sure your Zebra testnet node is running");
    println!("   and accessible at {}", client.config.rpc_endpoint);
    
    Ok(())
} 
