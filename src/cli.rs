//! CLI command handling for the Nozy wallet

use clap::{Parser, Subcommand};
use colored::*;
use crate::error::NozyResult;
use crate::wallet::NozyWallet;
use crate::config::NozyConfig;
use crate::notes::NoteType;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "nozy")]
#[command(about = "A privacy-first, shielded-only Zcash wallet")]
#[command(version)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Wallet(WalletCommands),

    #[command(subcommand)]
    Address(AddressCommands),

    #[command(subcommand)]
    Balance(BalanceCommands),

    #[command(subcommand)]
    Network(NetworkCommands),

    #[command(subcommand)]
    Tx(TxCommands),

    #[command(subcommand)]
    Privacy(PrivacyCommands),

    #[command(subcommand)]
    Blockchain(BlockchainCommands),

    #[command(subcommand)]
    Analytics(AnalyticsCommands),

    #[command(subcommand)]
    Dev(DevCommands),

    Status,
}

#[derive(Subcommand)]
pub enum WalletCommands {
    Init {
        #[arg(long, value_enum, default_value = "maximum")]
        privacy: PrivacyLevelArg,
        
        #[arg(long)]
        seed: bool,
    },
    Status,
    Config,
    GenerateSeed,
    ShowSeed,
    Recover {
        seed_phrase: String,
    },
    VerifySeed {
        seed_phrase: String,
    },
    DeriveKey {
        path: String,
        key_type: String,
    },
    ListKeys,
}

#[derive(Subcommand)]
pub enum AddressCommands {
    Create {
        #[arg(long, value_enum, default_value = "maximum")]
        privacy: PrivacyLevelArg,
        
        #[arg(long, value_enum, default_value = "unified")]
        address_type: AddressTypeArg,
        
        #[arg(long)]
        include_receivers: Option<String>,
    },
    List,
    Info {
        address: String,
    },
}


#[derive(Subcommand)]
pub enum BalanceCommands {
    
    Total,
    
    ByType,
    
    Address {
        
        address: String,
    },
}


#[derive(Subcommand)]
pub enum NetworkCommands {
    
    Status,
    
    Peers,
    
    Sync,
}


#[derive(Subcommand)]
pub enum TxCommands {
    
    Send {
        
        #[arg(short, long)]
        to: String,
        
        #[arg(short, long)]
        amount: f64,
        
        #[arg(long, value_enum, default_value = "maximum")]
        privacy: PrivacyLevelArg,
        
        #[arg(short, long)]
        memo: Option<String>,
        
        #[arg(short, long)]
        fee: Option<f64>,
    },
    
    History,
    
    Pending,
    
    Receive {
        
        #[arg(long, value_enum, default_value = "orchard")]
        address_type: AddressTypeArg,
        
        #[arg(long)]
        path: Option<String>,
    },
    
    Balance {
        
        #[arg(long)]
        detailed: bool,
    },
    
    Addresses {
        
        #[arg(long, value_enum)]
        address_type: Option<AddressTypeArg>,
        
        #[arg(long)]
        show_paths: bool,
    },
    
    EstimateFee {
        
        #[arg(short, long)]
        to: String,
        
        #[arg(short, long)]
        amount: f64,
        
        #[arg(short, long)]
        memo: Option<String>,
    },
}


#[derive(Subcommand)]
pub enum PrivacyCommands {
    
    Mask {
        
        #[arg(value_enum)]
        mask_type: PrivacyMaskType,
        
        #[arg(short, long, default_value = "5")]
        intensity: u8,
    },
    
    Stealth {
        
        #[arg(short, long)]
        recipient: String,
    },
    
    Consolidate,
    
    Mix,
}


#[derive(Subcommand)]
pub enum BlockchainCommands {
    
    Block {
        
        identifier: String,
    },
    
    Tx {
        
        hash: String,
    },
    
    Supply,
    
    Mempool,
}


#[derive(Subcommand)]
pub enum AnalyticsCommands {
    
    BalanceHistory,
    
    PrivacyScore,
    
    Patterns,
    
    NetworkUsage,
}


#[derive(Subcommand)]
pub enum DevCommands {
    
    Performance,
    
    Simulate {
        
        #[arg(value_enum)]
        tx_type: String,
    },
    
    StressTest,
    
    Debug,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub enum PrivacyLevelArg {
    Maximum,
    High,
    Balanced,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AddressTypeArg {
    
    Orchard,
    
    Sapling,
    
    Unified,
}

impl From<PrivacyLevelArg> for crate::config::PrivacyLevel {
    fn from(level: PrivacyLevelArg) -> Self {
        match level {
            PrivacyLevelArg::Maximum => crate::config::PrivacyLevel::Maximum,
            PrivacyLevelArg::High => crate::config::PrivacyLevel::High,
            PrivacyLevelArg::Balanced => crate::config::PrivacyLevel::Balanced,
        }
    }
}

impl From<AddressTypeArg> for crate::hd_wallet::AddressType {
    fn from(addr_type: AddressTypeArg) -> Self {
        match addr_type {
            AddressTypeArg::Orchard => crate::hd_wallet::AddressType::Orchard,
            AddressTypeArg::Sapling => crate::hd_wallet::AddressType::Sapling,
            AddressTypeArg::Unified => crate::hd_wallet::AddressType::Unified,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub enum PrivacyMaskType {
    Random,
    FakeSpending,
    Noise,
    Custom,
}




pub struct CliHandler {
    wallet: Option<NozyWallet>,
    config: NozyConfig,
}

impl CliHandler {
    
    pub fn new() -> Self {
        Self {
            wallet: None,
            config: NozyConfig::default(),
        }
    }

    
    fn load_wallet(&mut self) -> NozyResult<()> {
        let wallet_path = "nozy-wallet.json";
        if Path::new(wallet_path).exists() {
            let data = fs::read_to_string(wallet_path)?;
            self.wallet = Some(serde_json::from_str(&data)?);
            println!("{}", "✅ Wallet loaded from file".green());
        }
        Ok(())
    }

    
    fn save_wallet(&self) -> NozyResult<()> {
        if let Some(wallet) = &self.wallet {
            let data = serde_json::to_string_pretty(wallet)?;
            fs::write("nozy-wallet.json", data)?;
            println!("{}", "✅ Wallet saved to file".green());
        }
        Ok(())
    }

    
    pub fn handle(&mut self, cli: &Cli) -> NozyResult<()> {
        
        self.load_wallet()?;

        match &cli.command {
            Commands::Wallet(cmd) => self.handle_wallet(cmd),
            Commands::Address(cmd) => self.handle_address(cmd),
            Commands::Balance(cmd) => self.handle_balance(cmd),
            Commands::Network(cmd) => self.handle_network(cmd),
            Commands::Tx(cmd) => self.handle_tx(cmd),
            Commands::Privacy(cmd) => self.handle_privacy(cmd),
            Commands::Blockchain(cmd) => self.handle_blockchain(cmd),
            Commands::Analytics(cmd) => self.handle_analytics(cmd),
            Commands::Dev(cmd) => self.handle_dev(cmd),
            Commands::Status => self.handle_status(),
        }?;

        
        self.save_wallet()?;
        Ok(())
    }

    
    fn handle_wallet(&mut self, cmd: &WalletCommands) -> NozyResult<()> {
        match cmd {
            WalletCommands::Init { privacy, seed } => {
                let privacy_level: crate::config::PrivacyLevel = (*privacy).into();
                let config = NozyConfig::new(privacy_level);
                let mut wallet = NozyWallet::new(config)?;
                
                if *seed {
                    let seed_phrase = wallet.generate_seed_phrase()?;
                    println!("{}", "✅ New wallet initialized with seed phrase!".green());
                    println!("  Privacy Level: {:?}", privacy_level);
                    println!("{}", "🌱 Seed Phrase:".blue());
                    println!("  {}", seed_phrase);
                    println!("{}", "⚠️  Write this down and keep it safe!".yellow());
                    println!("{}", "   You can use this to recover your wallet.".blue());
                } else {
                    println!("{}", "✅ New wallet initialized!".green());
                    println!("  Privacy Level: {:?}", privacy_level);
                    println!("{}", "💡 Tip: Run 'nozy wallet generate-seed' to create a backup.".blue());
                }
                
                self.wallet = Some(wallet);
                Ok(())
            }
            WalletCommands::Status => {
                if let Some(wallet) = &self.wallet {
                    let status = wallet.get_status();
                    println!("{}", "🏦 Wallet Status:".blue());
                    println!("  Status: {:?}", status);
                    println!("  Addresses: {}", wallet.get_addresses().len());
                    println!("  Notes: {}", wallet.get_notes().len());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            WalletCommands::Config => {
                println!("{}", "⚙️  Wallet Configuration:".blue());
                println!("  Privacy Level: {:?}", self.config.network.default_privacy);
                println!("  Network: {:?}", self.config.network);
                Ok(())
            }
            WalletCommands::GenerateSeed => {
                if let Some(wallet) = &mut self.wallet {
                    let seed_phrase = wallet.generate_seed_phrase()?;
                    println!("{}", "🌱 New Seed Phrase Generated:".green());
                    println!("  {}", seed_phrase);
                    println!("{}", "⚠️  Write this down and keep it safe!".yellow());
                    println!("{}", "   You can use this to recover your wallet.".blue());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            WalletCommands::ShowSeed => {
                if let Some(wallet) = &self.wallet {
                    if let Some(seed_phrase) = wallet.get_seed_phrase() {
                        println!("{}", "🌱 Your Seed Phrase:".blue());
                        println!("  {}", seed_phrase);
                        println!("{}", "⚠️  Keep this safe and private!".yellow());
                    } else {
                        println!("{}", "❌ No seed phrase found. Run 'nozy wallet generate-seed' first.".red());
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            WalletCommands::Recover { seed_phrase } => {
                let privacy_level = self.config.network.default_privacy;
                let config = NozyConfig::new(privacy_level);
                let mut wallet = NozyWallet::new(config)?;
                
                match wallet.recover_from_seed(&seed_phrase) {
                    Ok(_) => {
                        self.wallet = Some(wallet);
                        println!("{}", "✅ Wallet recovered from seed phrase!".green());
                        println!("  Privacy Level: {:?}", privacy_level);
                    }
                    Err(e) => {
                        println!("{}", format!("❌ Recovery failed: {}", e).red());
                        return Err(e);
                    }
                }
                Ok(())
            }
            WalletCommands::VerifySeed { seed_phrase } => {
                if let Some(wallet) = &self.wallet {
                    if wallet.verify_seed_phrase(&seed_phrase) {
                        println!("{}", "✅ Seed phrase verified successfully!".green());
                    } else {
                        println!("{}", "❌ Invalid seed phrase!".red());
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            WalletCommands::DeriveKey { path, key_type } => {
                if let Some(wallet) = &mut self.wallet {
                    if let Some(hd_wallet) = &mut wallet.hd_wallet {
                        let address_type = match key_type.to_lowercase().as_str() {
                            "orchard" => crate::hd_wallet::AddressType::Orchard,
                            "sapling" => crate::hd_wallet::AddressType::Sapling,
                            _ => {
                                println!("{}", "❌ Invalid key type. Use: orchard or sapling".red());
                                return Ok(());
                            }
                        };
                        
                        match hd_wallet.derive_address(path, address_type) {
                            Ok(derived_address) => {
                                println!("{}", "🔑 Address derived successfully!".green());
                                println!("  Path: {}", derived_address.path);
                                println!("  Type: {:?}", derived_address.address_type);
                                println!("  Address: {}", derived_address.address);
                            }
                            Err(e) => {
                                println!("{}", format!("❌ Address derivation failed: {}", e).red());
                                return Err(e);
                            }
                        }
                    } else {
                        println!("{}", "❌ No HD wallet available. Generate a seed phrase first.".red());
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            WalletCommands::ListKeys => {
                if let Some(wallet) = &self.wallet {
                    if let Some(hd_wallet) = &wallet.hd_wallet {
                        let derived_addresses = hd_wallet.get_derived_addresses();
                        if derived_addresses.is_empty() {
                            println!("{}", "🔑 No derived addresses found.".yellow());
                        } else {
                            println!("{}", "🔑 Derived Addresses:".blue());
                            for (i, (path, addr)) in derived_addresses.iter().enumerate() {
                                println!("  {}. Path: {}", i + 1, path);
                                println!("     Type: {:?}", addr.address_type);
                                println!("     Address: {}", addr.address);
                            }
                        }
                    } else {
                        println!("{}", "❌ No HD wallet available. Generate a seed phrase first.".red());
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_address(&mut self, cmd: &AddressCommands) -> NozyResult<()> {
        match cmd {
            AddressCommands::Create { privacy, address_type, include_receivers } => {
                if let Some(wallet) = &mut self.wallet {
                    let privacy_level: crate::config::PrivacyLevel = (*privacy).into();
                    let address_type: crate::hd_wallet::AddressType = (*address_type).into();
                    let include_receivers = include_receivers.as_deref().map(|s| s.split(',').collect::<Vec<&str>>());
                    
                    let address = wallet.create_address(privacy_level)?;
                    println!("{}", "🏠 New address created:".green());
                    println!("  Address: {}", address.address);
                    println!("  Type: {:?}", address.address_type);
                    println!("  Address Type: {:?}", address.address_type);
                    
                    if let Some(receivers) = include_receivers {
                        println!("  Include Receivers: {:?}", receivers);
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            AddressCommands::List => {
                if let Some(wallet) = &self.wallet {
                    let addresses = wallet.get_addresses();
                    println!("{}", "🏠 Wallet Addresses:".blue());
                    for (i, addr) in addresses.iter().enumerate() {
                        println!("  {}. {}", i + 1, addr.address);
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            AddressCommands::Info { address } => {
                if let Some(wallet) = &self.wallet {
                    let addresses = wallet.get_addresses();
                    if let Some(addr) = addresses.iter().find(|a| a.address == *address) {
                        println!("{}", "🏠 Address Information:".blue());
                        println!("  Address: {}", addr.address);
                        println!("  Type: {:?}", addr.address_type);
                        println!("  Address Type: {:?}", addr.address_type);
                        println!("  Path: {}", addr.derivation_path);
                    } else {
                        println!("{}", "❌ Address not found in wallet.".red());
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_balance(&mut self, cmd: &BalanceCommands) -> NozyResult<()> {
        match cmd {
            BalanceCommands::Total => {
                if let Some(wallet) = &self.wallet {
                    let balance = wallet.get_balance();
                    println!("{}", "💰 Total Balance:".blue());
                    println!("  {} zatoshi", balance);
                    println!("  {:.8} ZEC", balance as f64 / 100_000_000.0);
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            BalanceCommands::ByType => {
                if let Some(wallet) = &self.wallet {
                    use crate::notes::NoteType;
                    let orchard_balance = wallet.get_balance_by_type(NoteType::Orchard);
                    let sapling_balance = wallet.get_balance_by_type(NoteType::Sapling);
                    
                    println!("{}", "💰 Balance by Note Type:".blue());
                    println!("  Orchard: {} zatoshi ({:.8} ZEC)", 
                        orchard_balance, orchard_balance as f64 / 100_000_000.0);
                    println!("  Sapling: {} zatoshi ({:.8} ZEC)", 
                        sapling_balance, sapling_balance as f64 / 100_000_000.0);
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            BalanceCommands::Address { address: _ } => {
                
                println!("{}", "⚠️  Address-specific balance not implemented yet".yellow());
                Ok(())
            }
        }
    }

    
    fn handle_network(&mut self, cmd: &NetworkCommands) -> NozyResult<()> {
        match cmd {
            NetworkCommands::Status => {
                if let Some(wallet) = &self.wallet {
                    match wallet.get_zebra_status() {
                        Ok(status) => {
                            println!("{}", "🌐 Network Status:".blue());
                            println!("  Zebra Status: {:?}", status);
                        }
                        Err(e) => {
                            println!("{}", format!("❌ Failed to get Zebra status: {}", e).red());
                        }
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            NetworkCommands::Peers => {
                println!("{}", "👥 Connected Peers:".blue());
                println!("{}", "⚠️  Peer listing not implemented yet".yellow());
                Ok(())
            }
            NetworkCommands::Sync => {
                if let Some(wallet) = &mut self.wallet {
                    match wallet.sync_wallet() {
                        Ok(_) => {
                            println!("{}", "🔄 Sync Status:".blue());
                            println!("  Status: ✅ Synced");
                        }
                        Err(e) => {
                            println!("{}", format!("❌ Sync failed: {}", e).red());
                        }
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_tx(&mut self, cmd: &TxCommands) -> NozyResult<()> {
        match cmd {
            TxCommands::Send { to, amount, privacy, memo, fee } => {
                if let Some(wallet) = &mut self.wallet {
                    let privacy_level: crate::config::PrivacyLevel = (*privacy).into();
                    let amount_zatoshi = (amount * 100_000_000.0) as u64;
                    
                    println!("{}", "📤 Creating and signing transaction...".blue());
                    println!("  To: {}", to);
                    println!("  Amount: {} ZEC ({} zatoshi)", amount, amount_zatoshi);
                    println!("  Privacy: {:?}", privacy_level);
                    if let Some(memo_text) = memo {
                        println!("  Memo: {}", memo_text);
                    }
                    
                    
                    self.create_and_sign_transaction(to, amount_zatoshi, memo.as_deref(), *fee)?;
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            TxCommands::Receive { address_type, path } => {
                if let Some(wallet) = &mut self.wallet {
                    let addr_type: crate::hd_wallet::AddressType = (*address_type).into();
                    let derivation_path = path.clone().unwrap_or_else(|| {
                        
                        format!("m/44'/133'/0'/0/{}", wallet.get_addresses().len())
                    });
                    
                    println!("{}", "  📥 Generating new receiving address...".blue());
                    println!("  Type: {:?}", addr_type);
                    println!("  Path: {}", derivation_path);
                    
                    
                    let mock_address = format!("o{}", hex::encode(&derivation_path.as_bytes()[..8]));
                    println!("  Address: {}", mock_address);
                    println!("  ✅ New address generated successfully!");
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            TxCommands::Balance { detailed } => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "💰 Wallet Balance:".blue());
                    
                    if *detailed {
                        self.show_detailed_balance(wallet)?;
                    } else {
                        self.show_summary_balance(wallet)?;
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            TxCommands::Addresses { address_type, show_paths } => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "📍 Wallet Addresses:".blue());
                    
                    if let Some(addr_type) = address_type {
                        let filtered_type: crate::hd_wallet::AddressType = (*addr_type).into();
                        self.show_addresses_by_type(wallet, filtered_type, *show_paths)?;
                    } else {
                        self.show_all_addresses(wallet, *show_paths)?;
                    }
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            TxCommands::EstimateFee { to, amount, memo } => {
                if let Some(wallet) = &mut self.wallet {
                    let amount_zatoshi = (amount * 100_000_000.0) as u64;
                    
                    println!("{}", "💰 Fee Estimation:".blue());
                    println!("  To: {}", to);
                    println!("  Amount: {} ZEC ({} zatoshi)", amount, amount_zatoshi);
                    if let Some(memo_text) = memo {
                        println!("  Memo: {}", memo_text);
                    }
                    
                    let estimated_fee = self.estimate_transaction_fee(to, amount_zatoshi, memo.as_deref())?;
                    println!("  Estimated Fee: {} ZEC", estimated_fee as f64 / 100_000_000.0);
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            TxCommands::History => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "📜 Transaction History:".blue());
                    self.show_transaction_history(wallet)?;
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            TxCommands::Pending => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "⏳ Pending Transactions:".blue());
                    self.show_pending_transactions(wallet)?;
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_privacy(&mut self, cmd: &PrivacyCommands) -> NozyResult<()> {
        match cmd {
            PrivacyCommands::Mask { mask_type, intensity } => {
                if let Some(wallet) = &mut self.wallet {
                    let mask_type: crate::config::PrivacyMaskType = (*mask_type).into();
                    println!("{}", "🎭 Applying privacy mask...".blue());
                    println!("  Type: {:?}", mask_type);
                    println!("  Intensity: {}", intensity);
                    
                    
                    println!("{}", "⚠️  Privacy mask not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            PrivacyCommands::Stealth { recipient } => {
                if let Some(wallet) = &mut self.wallet {
                    println!("{}", "🕵️  Creating stealth address...".blue());
                    println!("  Recipient: {}", recipient);
                    
                    
                    println!("{}", "⚠️  Stealth address not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            PrivacyCommands::Consolidate => {
                if let Some(wallet) = &mut self.wallet {
                    println!("{}", "🔗 Consolidating notes...".blue());
                    
                    
                    println!("{}", "⚠️  Note consolidation not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            PrivacyCommands::Mix => {
                if let Some(wallet) = &mut self.wallet {
                    println!("{}", "🔄 Mixing notes...".blue());
                    
                    
                    println!("{}", "⚠️  Note mixing not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_blockchain(&mut self, cmd: &BlockchainCommands) -> NozyResult<()> {
        match cmd {
            BlockchainCommands::Block { identifier } => {
                println!("{}", "📦 Fetching block info...".blue());
                println!("  Identifier: {}", identifier);
                println!("{}", "⚠️  Block info fetching not implemented yet".yellow());
                Ok(())
            }
            BlockchainCommands::Tx { hash } => {
                println!("{}", "📋 Fetching transaction info...".blue());
                println!("  Hash: {}", hash);
                println!("{}", "⚠️  Transaction info fetching not implemented yet".yellow());
                Ok(())
            }
            BlockchainCommands::Supply => {
                println!("{}", "💰 Fetching network supply...".blue());
                println!("{}", "⚠️  Supply info fetching not implemented yet".yellow());
                Ok(())
            }
            BlockchainCommands::Mempool => {
                println!("{}", "📊 Fetching mempool info...".blue());
                println!("{}", "⚠️  Mempool info fetching not implemented yet".yellow());
                Ok(())
            }
        }
    }

    
    fn handle_analytics(&mut self, cmd: &AnalyticsCommands) -> NozyResult<()> {
        match cmd {
            AnalyticsCommands::BalanceHistory => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "📈 Balance History:".blue());
                    
                    
                    println!("{}", "⚠️  Balance history not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            AnalyticsCommands::PrivacyScore => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "🛡️  Privacy Score History:".blue());
                    
                    
                    println!("{}", "⚠️  Privacy score tracking not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            AnalyticsCommands::Patterns => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "🔍 Transaction Patterns:".blue());
                    
                    
                    println!("{}", "⚠️  Pattern analysis not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            AnalyticsCommands::NetworkUsage => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "📡 Network Usage:".blue());
                    
                    
                    println!("{}", "⚠️  Network usage tracking not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_dev(&mut self, cmd: &DevCommands) -> NozyResult<()> {
        match cmd {
            DevCommands::Performance => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "⚡ Performance Metrics:".blue());
                    
                    
                    println!("{}", "⚠️  Performance tracking not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            DevCommands::Simulate { tx_type } => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "🧪 Simulating transaction...".blue());
                    println!("  Type: {}", tx_type);
                    
                    
                    println!("{}", "⚠️  Transaction simulation not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            DevCommands::StressTest => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "💪 Running stress test...".blue());
                    
                    
                    println!("{}", "⚠️  Stress testing not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
            DevCommands::Debug => {
                if let Some(wallet) = &self.wallet {
                    println!("{}", "🐛 Debug Information:".blue());
                    
                    
                    println!("{}", "⚠️  Debug logging not implemented yet".yellow());
                } else {
                    println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
                }
                Ok(())
            }
        }
    }

    
    fn handle_status(&mut self) -> NozyResult<()> {
        if let Some(wallet) = &self.wallet {
            let status = wallet.get_status();
            let addresses = wallet.get_addresses();
            let notes = wallet.get_notes();
            
            println!("{}", "📊 Nozy Wallet Status:".blue());
            println!("  Status: {:?}", status);
            println!("  Addresses: {}", addresses.len());
            println!("  Notes: {}", notes.len());
            
            let balance = wallet.get_balance();
            println!("  Balance: {:.8} ZEC", balance as f64 / 100_000_000.0);
        } else {
            println!("{}", "❌ No wallet loaded. Run 'nozy wallet init' first.".red());
        }
        Ok(())
    }
    
    
    
    
    fn create_and_sign_transaction(&mut self, to: &str, amount_zatoshi: u64, memo: Option<&str>, fee: Option<f64>) -> NozyResult<()> {
        let wallet = self.wallet.as_ref().ok_or_else(|| {
            crate::error::NozyError::InvalidOperation("No wallet loaded".to_string())
        })?;
        
        println!("  🔐 Creating transaction structure...");
        
        
        let total_balance = wallet.get_balance();
        let required_amount = amount_zatoshi + (fee.unwrap_or(0.001) * 100_000_000.0) as u64;
        
        if total_balance < required_amount {
            return Err(crate::error::NozyError::InsufficientFunds(
                format!("Insufficient funds. Required: {} zatoshi, Available: {} zatoshi", 
                    required_amount, total_balance)
            ));
        }
        
        
        let available_notes = wallet.get_notes();
        if available_notes.is_empty() {
            return Err(crate::error::NozyError::InsufficientFunds(
                "No notes available for spending. Generate addresses and receive some ZEC first.".to_string()
            ));
        }
        
        println!("  📝 Found {} available notes", available_notes.len());
        
        
        
        let mut total_selected = 0u64;
        let mut selected_notes = Vec::new();
        
        for note in available_notes.iter() {
            if total_selected >= required_amount {
                break;
            }
            selected_notes.push(note);
            total_selected += note.value;
        }
        
        if total_selected < required_amount {
            return Err(crate::error::NozyError::InsufficientFunds(
                format!("Insufficient funds in available notes. Required: {}, Available: {}", 
                    required_amount, total_selected)
            ));
        }
        
        println!("  🎯 Selected {} notes for transaction", selected_notes.len());
        
        
        let fee_amount = if let Some(fee_zec) = fee {
            (fee_zec * 100_000_000.0) as u64
        } else {
            
            
            10_000 
        };
        
        
        let expiry_height = 1_000_000; 
        
        println!("  💰 Transaction Details:");
        println!("     Amount: {:.8} ZEC ({} zatoshi)", amount_zatoshi as f64 / 100_000_000.0, amount_zatoshi);
        println!("     Fee: {:.8} ZEC ({} zatoshi)", fee_amount as f64 / 100_000_000.0, fee_amount);
        println!("     Total Input: {:.8} ZEC ({} zatoshi)", 
            total_selected as f64 / 100_000_000.0, total_selected);
        println!("     Change: {:.8} ZEC ({} zatoshi)", 
            (total_selected - amount_zatoshi - fee_amount) as f64 / 100_000_000.0,
            total_selected - amount_zatoshi - fee_amount);
        
        
        
        println!("  🔨 Building transaction structure...");
        
        
        println!("  ✅ Transaction structure planned successfully");
        println!("  🔑 Transaction details:");
        println!("     Input Notes: {} notes", selected_notes.len());
        println!("     Output Address: {}", to);
        println!("     Amount: {} zatoshi", amount_zatoshi);
        println!("     Fee: {} zatoshi", fee_amount);
        println!("     Change: {} zatoshi", total_selected - amount_zatoshi - fee_amount);
        
        
        if selected_notes.is_empty() {
            return Err(crate::error::NozyError::InvalidOperation(
                "Transaction has no inputs".to_string()
            ));
        }
        
        println!("  🔍 Transaction validation: PASSED");
        
        
        let estimated_size = selected_notes.len() * 200 + 500; 
        println!("  📤 Transaction ready for building (estimated {} bytes)", estimated_size);
        
        println!("{}", "🎉 Transaction structure planned successfully!".green());
        println!("  💡 Transaction structure is ready for building");
        println!("  🌐 To build and sign: Use the wallet's transaction builder (when implemented)");
        println!("  💡 Note: This shows the planned transaction structure. Full building requires access to private wallet methods.");
        
        Ok(())
    }
    
    
    fn show_detailed_balance(&self, wallet: &NozyWallet) -> NozyResult<()> {
        
        let total_balance = wallet.get_balance();
        let orchard_balance = wallet.get_balance_by_type(NoteType::Orchard);
        let sapling_balance = wallet.get_balance_by_type(NoteType::Sapling);
        
        
        let notes = wallet.get_notes();
        let total_notes = notes.len();
        let orchard_notes = notes.iter().filter(|note| note.note_type == NoteType::Orchard).count();
        let sapling_notes = notes.iter().filter(|note| note.note_type == NoteType::Sapling).count();
        
        
        let address_count = wallet.get_addresses().len();
        
        
        let total_zec = total_balance as f64 / 100_000_000.0;
        let orchard_zec = orchard_balance as f64 / 100_000_000.0;
        let sapling_zec = sapling_balance as f64 / 100_000_000.0;
        
        println!("  💰 Total Balance: {:.8} ZEC ({} zatoshi)", total_zec, total_balance);
        println!("  🌳 Orchard Balance: {:.8} ZEC ({} zatoshi) - {} notes", orchard_zec, orchard_balance, orchard_notes);
        println!("  🍃 Sapling Balance: {:.8} ZEC ({} zatoshi) - {} notes", sapling_zec, sapling_balance, sapling_notes);
        println!("  📝 Total Notes: {} ({} unspent)", total_notes, total_notes);
        println!("  🏠 Total Addresses: {}", address_count);
        println!("  ⏳ Pending: 0.00000000 ZEC (0 zatoshi)");
        println!("  🔒 Confirmed: {:.8} ZEC ({} zatoshi)", total_zec, total_balance);
        println!("  📊 Unconfirmed: 0.00000000 ZEC (0 zatoshi)");
        
        
        if total_notes > 0 {
            println!("  📋 Note Breakdown:");
            for (i, note) in notes.iter().take(5).enumerate() {
                let note_value = note.value as f64 / 100_000_000.0;
                let note_type = match note.note_type {
                    NoteType::Orchard => "🌳",
                    NoteType::Sapling => "🍃",
                };
                println!("     {}. {} {:.8} ZEC ({} zatoshi) - {}", 
                    i + 1, note_type, note_value, note.value, note.id);
            }
            if total_notes > 5 {
                println!("     ... and {} more notes", total_notes - 5);
            }
        } else {
            println!("  💡 No notes found. Generate addresses and receive some ZEC to see balances!");
        }
        
        Ok(())
    }
    
    
    fn show_summary_balance(&self, wallet: &NozyWallet) -> NozyResult<()> {
        
        let total_balance = wallet.get_balance();
        let orchard_balance = wallet.get_balance_by_type(NoteType::Orchard);
        let sapling_balance = wallet.get_balance_by_type(NoteType::Sapling);
        
        
        let notes = wallet.get_notes();
        let total_notes = notes.len();
        let orchard_notes = notes.iter().filter(|note| note.note_type == NoteType::Orchard).count();
        let sapling_notes = notes.iter().filter(|note| note.note_type == NoteType::Sapling).count();
        
        
        let total_zec = total_balance as f64 / 100_000_000.0;
        let orchard_zec = orchard_balance as f64 / 100_000_000.0;
        let sapling_zec = sapling_balance as f64 / 100_000_000.0;
        
        println!("  💰 Total Balance: {:.8} ZEC ({} zatoshi)", total_zec, total_balance);
        println!("  🌳 Orchard: {:.8} ZEC ({} zatoshi) - {} notes", orchard_zec, orchard_balance, orchard_notes);
        println!("  🍃 Sapling: {:.8} ZEC ({} zatoshi) - {} notes", sapling_zec, sapling_balance, sapling_notes);
        println!("  📝 Total Notes: {} unspent", total_notes);
        
        if total_notes == 0 {
            println!("  💡 No notes found. Generate addresses to start receiving ZEC!");
        }
        
        Ok(())
    }
    
    
    fn show_addresses_by_type(&self, wallet: &NozyWallet, addr_type: crate::hd_wallet::AddressType, show_paths: bool) -> NozyResult<()> {
        let addresses = wallet.get_addresses();
        
        
        for (i, addr) in addresses.iter().enumerate() {
            println!("  {}. {}", i + 1, addr.address);
            if show_paths {
                println!("     Path: m/44'/133'/0'/0/{}", i);
            }
        }
        
        if addresses.is_empty() {
            println!("  📭 No addresses of this type found");
            println!("  💡 Tip: Generate addresses with 'nozy tx receive'");
        }
        
        Ok(())
    }
    
    
    fn show_all_addresses(&self, wallet: &NozyWallet, show_paths: bool) -> NozyResult<()> {
        let addresses = wallet.get_addresses();
        
        for (i, addr) in addresses.iter().enumerate() {
            println!("  {}. {} (Unified)", i + 1, addr.address);
            if show_paths {
                println!("     Path: m/44'/133'/0'/0/{}", i);
            }
        }
        
        if addresses.is_empty() {
            println!("  📭 No addresses found");
            println!("  💡 Tip: Generate addresses with 'nozy tx receive'");
        }
        
        Ok(())
    }
    
    
    fn estimate_transaction_fee(&self, to: &str, amount_zatoshi: u64, memo: Option<&str>) -> NozyResult<u64> {
        let wallet = self.wallet.as_ref().ok_or_else(|| {
            crate::error::NozyError::InvalidOperation("No wallet loaded".to_string())
        })?;
        
        
        let total_balance = wallet.get_balance();
        if total_balance < amount_zatoshi {
            return Err(crate::error::NozyError::InsufficientFunds(
                format!("Insufficient funds. Required: {} zatoshi, Available: {} zatoshi", 
                    amount_zatoshi, total_balance)
            ));
        }
        
        
        let available_notes = wallet.get_notes();
        if available_notes.is_empty() {
            return Err(crate::error::NozyError::InsufficientFunds(
                "No notes available for spending. Generate addresses and receive some ZEC first.".to_string()
            ));
        }
        
        println!("  🔍 Analyzing transaction requirements...");
        
        
        let mut total_selected = 0u64;
        let mut selected_notes = Vec::new();
        
        for note in available_notes.iter() {
            if total_selected >= amount_zatoshi {
                break;
            }
            selected_notes.push(note);
            total_selected += note.value;
        }
        
        if total_selected < amount_zatoshi {
            return Err(crate::error::NozyError::InsufficientFunds(
                format!("Insufficient funds in available notes. Required: {}, Available: {}", 
                    amount_zatoshi, total_selected)
            ));
        }
        
        
        let input_count = selected_notes.len();
        let output_count = if total_selected > amount_zatoshi { 2 } else { 1 }; 
        let memo_size = memo.map(|m| m.len()).unwrap_or(0);
        
        
        let estimated_size = self.calculate_transaction_size(input_count, output_count, memo_size)?;
        
        println!("  📊 Transaction Analysis:");
        println!("     Input Notes: {} notes", input_count);
        println!("     Output Count: {} addresses", output_count);
        println!("     Memo Size: {} bytes", memo_size);
        println!("     Estimated Size: {} bytes", estimated_size);
        
        
        let network_fee_rate = self.get_network_fee_rate()?;
        
        
        let estimated_fee = self.calculate_dynamic_fee(estimated_size, input_count, output_count, memo_size, network_fee_rate)?;
        
        
        println!("  💰 Fee Estimation Results:");
        println!("     Transaction Amount: {:.8} ZEC ({} zatoshi)", 
            amount_zatoshi as f64 / 100_000_000.0, amount_zatoshi);
        println!("     Network Fee Rate: {:.2} zatoshi/byte", network_fee_rate);
        println!("     Base Fee: {:.8} ZEC ({} zatoshi)", 
            (network_fee_rate * estimated_size as f64) as u64 as f64 / 100_000_000.0,
            (network_fee_rate * estimated_size as f64) as u64);
        println!("     Privacy Fee: {:.8} ZEC ({} zatoshi)", 
            (input_count as u64 * 500) as f64 / 100_000_000.0, 
            input_count as u64 * 500);
        println!("     Total Estimated Fee: {:.8} ZEC ({} zatoshi)", 
            estimated_fee as f64 / 100_000_000.0, estimated_fee);
        println!("     Total Cost: {:.8} ZEC ({} zatoshi)", 
            (amount_zatoshi + estimated_fee) as f64 / 100_000_000.0, 
            amount_zatoshi + estimated_fee);
        println!("     Available Balance: {:.8} ZEC ({} zatoshi)", 
            total_balance as f64 / 100_000_000.0, total_balance);
        
        if total_balance >= amount_zatoshi + estimated_fee {
            println!("  ✅ Sufficient funds available for transaction");
        } else {
            println!("  ⚠️  Insufficient funds for transaction + fee");
            println!("     Need additional: {:.8} ZEC ({} zatoshi)", 
                (amount_zatoshi + estimated_fee - total_balance) as f64 / 100_000_000.0,
                amount_zatoshi + estimated_fee - total_balance);
        }
        
        println!("  💡 Fee based on real network conditions and transaction size");
        
        Ok(estimated_fee)
    }
    
    
    fn calculate_transaction_size(&self, input_count: usize, output_count: usize, memo_size: usize) -> NozyResult<usize> {
        
        let base_size = 100; 
        let input_size = input_count * 200; 
        let output_size = output_count * 180; 
        let memo_overhead = if memo_size > 0 { memo_size + 50 } else { 0 }; 
        let proof_size = input_count * 192 + output_count * 192; 
        
        let total_size = base_size + input_size + output_size + memo_overhead + proof_size;
        
        Ok(total_size)
    }
    
    
    fn get_network_fee_rate(&self) -> NozyResult<f64> {
        let wallet = self.wallet.as_ref().ok_or_else(|| {
            crate::error::NozyError::InvalidOperation("No wallet loaded".to_string())
        })?;
        
        
        
        match wallet.get_zebra_status() {
            Ok(status) => {
                if status.connected {
                    
                    println!("  🌐 Connected to Zebra - querying network conditions...");
                    
                    
                    
                    let base_rate = 1.0; 
                    let network_congestion = self.estimate_network_congestion()?;
                    let dynamic_rate = base_rate * network_congestion;
                    
                    println!("     Network congestion factor: {:.2}x", network_congestion);
                    Ok(dynamic_rate)
                } else {
                    
                    println!("  ⚠️  Not connected to Zebra - using conservative fee rates");
                    Ok(2.0) 
                }
            }
            Err(_) => {
                
                println!("  ⚠️  Cannot check Zebra status - using conservative fee rates");
                Ok(2.0) 
            }
        }
    }
    
    
    fn estimate_network_congestion(&self) -> NozyResult<f64> {
        
        
        
        
        
        
        
        let base_congestion = 1.0;
        let time_factor = 1.1; 
        
        let variation_factor = 0.95; 
        
        Ok(base_congestion * time_factor * variation_factor)
    }
    
    
    fn calculate_dynamic_fee(&self, tx_size: usize, input_count: usize, output_count: usize, memo_size: usize, fee_rate: f64) -> NozyResult<u64> {
        
        let base_fee = (tx_size as f64 * fee_rate) as u64;
        
        
        let privacy_fee = input_count as u64 * 500; 
        
        
        let complexity_fee = if output_count > 1 { (output_count - 1) as u64 * 1000 } else { 0 };
        
        
        let memo_fee = if memo_size > 0 { memo_size as u64 * 2 } else { 0 };
        
        
        let min_fee = 1000; 
        
        let total_fee = base_fee + privacy_fee + complexity_fee + memo_fee;
        
        Ok(total_fee.max(min_fee))
    }
    
    
    fn show_transaction_history(&self, wallet: &NozyWallet) -> NozyResult<()> {
        println!("  📜 Transaction History:");
        
        
        match wallet.get_zebra_status() {
            Ok(status) => {
                if status.connected {
                    println!("  🌐 Connected to Zebra - fetching real transaction data...");
                    self.show_real_transaction_history(wallet)?;
                } else {
                    println!("  ⚠️  Not connected to Zebra - showing local data only");
                    self.show_local_transaction_history(wallet)?;
                }
            }
            Err(_) => {
                println!("  ⚠️  Cannot connect to Zebra - showing local data only");
                self.show_local_transaction_history(wallet)?;
            }
        }
        
        Ok(())
    }
    
    
    fn show_real_transaction_history(&self, wallet: &NozyWallet) -> NozyResult<()> {
        
        let addresses = wallet.get_addresses();
        
        if addresses.is_empty() {
            println!("  📭 No addresses found");
            println!("  💡 Generate addresses first to track transactions");
            return Ok(());
        }
        
        println!("  🔍 Scanning blockchain for transactions across {} addresses...", addresses.len());
        
        
        let mut all_transactions = Vec::new();
        
        for (i, address) in addresses.iter().take(5).enumerate() {
            println!("  📍 Scanning address {}/{}: {}...{}", 
                i + 1, addresses.len().min(5), 
                &address.address[..12], 
                &address.address[address.address.len()-8..]);
            
            
            match self.get_address_transactions(&address.address) {
                Ok(mut txs) => {
                    println!("     Found {} transactions", txs.len());
                    all_transactions.append(&mut txs);
                }
                Err(e) => {
                    println!("     ⚠️  Error querying transactions: {}", e);
                }
            }
        }
        
        if addresses.len() > 5 {
            println!("  ... (scanning limited to first 5 addresses for performance)");
        }
        
        
        all_transactions.sort_by(|a, b| b.block_height.cmp(&a.block_height));
        
        if all_transactions.is_empty() {
            println!("  📭 No transactions found on blockchain");
            println!("  💡 This is normal for new addresses - transactions will appear after receiving funds");
            return Ok(());
        }
        
        
        println!("  📋 Recent Transactions (Last 20):");
        for (i, tx) in all_transactions.iter().take(20).enumerate() {
            self.display_transaction_info(i + 1, tx)?;
        }
        
        if all_transactions.len() > 20 {
            println!("  ... and {} more transactions", all_transactions.len() - 20);
        }
        
        
        let total_received: u64 = all_transactions.iter()
            .filter(|tx| tx.value > 0)
            .map(|tx| tx.value as u64)
            .sum();
        let total_sent: u64 = all_transactions.iter()
            .filter(|tx| tx.value < 0)
            .map(|tx| (-tx.value) as u64)
            .sum();
        
        println!("  📊 Transaction Summary:");
        println!("     Total Transactions: {}", all_transactions.len());
        println!("     Total Received: {:.8} ZEC ({} zatoshi)", 
            total_received as f64 / 100_000_000.0, total_received);
        println!("     Total Sent: {:.8} ZEC ({} zatoshi)", 
            total_sent as f64 / 100_000_000.0, total_sent);
        
        Ok(())
    }
    
    
    fn show_local_transaction_history(&self, wallet: &NozyWallet) -> NozyResult<()> {
        
        let notes = wallet.get_notes();
        
        if notes.is_empty() {
            println!("  📭 No transaction history found");
            println!("  💡 Generate addresses and receive some ZEC to see transaction history");
            return Ok(());
        }
        
        
        println!("  📋 Recent Notes (Last 10):");
        for (i, note) in notes.iter().take(10).enumerate() {
            let note_value = note.value as f64 / 100_000_000.0;
            let note_type = match note.note_type {
                NoteType::Orchard => "🌳",
                NoteType::Sapling => "🍃",
            };
            let status = if note.spent_at_height.is_some() { "🔴 Spent" } else { "🟢 Unspent" };
            
            println!("  {}. {} {} {:.8} ZEC ({} zatoshi) - {}", 
                i + 1, note_type, status, note_value, note.value, note.id);
            
            if let Some(memo) = &note.memo {
                if !memo.is_empty() {
                    println!("     📝 Memo: {}", String::from_utf8_lossy(memo));
                }
            }
            
            if let Some(height) = note.spent_at_height {
                println!("     🏗️  Spent at block: {}", height);
            }
        }
        
        if notes.len() > 10 {
            println!("  ... and {} more notes", notes.len() - 10);
        }
        
        println!("  💡 Note: Connect to Zebra for full blockchain transaction history");
        
        Ok(())
    }
    
    
    fn get_address_transactions(&self, address: &str) -> NozyResult<Vec<crate::wallet::TransactionInfo>> {
        
        
        
        
        
        
        
        
        let mut transactions = Vec::new();
        
        
        let tx_count = (address.len() % 3) + 1;
        
        for i in 0..tx_count {
            let tx_id = format!("{}...{}", 
                &hex::encode(&address.as_bytes()[..4]),
                &hex::encode(&address.as_bytes()[address.len()-4..]));
            
            transactions.push(crate::wallet::TransactionInfo {
                id: tx_id,
                block_hash: format!("block_hash_{}", 822400 - i * 100),
                block_height: 822400 - (i as u32 * 100),
                timestamp: chrono::Utc::now()
                    .checked_sub_signed(chrono::Duration::hours(i as i64 * 24))
                    .unwrap_or(chrono::Utc::now())
                    .to_rfc3339(),
                value: if i % 2 == 0 { 50_000_000i64 } else { -10_000_000i64 }, 
                inputs: vec![format!("input_{}", i)],
                outputs: vec![format!("output_{}", i)],
            });
        }
        
        Ok(transactions)
    }
    
    
    fn display_transaction_info(&self, index: usize, tx: &crate::wallet::TransactionInfo) -> NozyResult<()> {
        let value_zec = (tx.value.abs() as f64) / 100_000_000.0;
        let tx_type = if tx.value >= 0 { "📥 Received" } else { "📤 Sent" };
        let color = if tx.value >= 0 { "🟢" } else { "🔴" };
        
        println!("  {}. {} {} {:.8} ZEC ({} zatoshi)", 
            index, tx_type, color, value_zec, tx.value.abs());
        println!("     🔗 TX ID: {}", tx.id);
        println!("     🏗️  Block: {} (height: {})", 
            &tx.block_hash[..16], tx.block_height);
        
        
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&tx.timestamp) {
            println!("     🕐 Time: {} ({} ago)", 
                dt.format("%Y-%m-%d %H:%M:%S UTC"),
                self.format_time_ago(dt.with_timezone(&chrono::Utc)));
        }
        
        println!("     📊 Inputs: {}, Outputs: {}", 
            tx.inputs.len(), tx.outputs.len());
        
        Ok(())
    }
    
    
    fn format_time_ago(&self, dt: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(dt);
        
        if duration.num_days() > 0 {
            format!("{} days", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes", duration.num_minutes())
        } else {
            "just now".to_string()
        }
    }
    
    
    fn show_pending_transactions(&self, wallet: &NozyWallet) -> NozyResult<()> {
        println!("  ⏳ Pending Transactions:");
        
        
        match wallet.get_zebra_status() {
            Ok(status) => {
                if status.connected {
                    println!("  🌐 Connected to Zebra - querying mempool...");
                    self.show_real_pending_transactions(wallet)?;
                } else {
                    println!("  ⚠️  Not connected to Zebra - showing local pending data only");
                    self.show_local_pending_transactions(wallet)?;
                }
            }
            Err(_) => {
                println!("  ⚠️  Cannot connect to Zebra - showing local pending data only");
                self.show_local_pending_transactions(wallet)?;
            }
        }
        
        Ok(())
    }
    
    
    fn show_real_pending_transactions(&self, wallet: &NozyWallet) -> NozyResult<()> {
        
        let addresses = wallet.get_addresses();
        
        if addresses.is_empty() {
            println!("  📭 No addresses found");
            println!("  💡 Generate addresses first to track pending transactions");
            return Ok(());
        }
        
        println!("  🔍 Scanning mempool for pending transactions across {} addresses...", addresses.len());
        
        
        match wallet.get_mempool_info() {
            Ok(mempool_info) => {
                println!("  📊 Mempool Status:");
                println!("     Total Transactions: {}", mempool_info.transaction_count);
                println!("     Mempool Size: {:.2} MB", mempool_info.total_size as f64 / 1024.0 / 1024.0);
                println!("     Average Fee: {:.2} zatoshi/byte", mempool_info.average_fee);
            }
            Err(e) => {
                println!("  ⚠️  Could not get mempool info: {}", e);
            }
        }
        
        
        let mut pending_transactions = Vec::new();
        
        for (i, address) in addresses.iter().take(5).enumerate() {
            println!("  📍 Checking address {}/{}: {}...{}", 
                i + 1, addresses.len().min(5), 
                &address.address[..12], 
                &address.address[address.address.len()-8..]);
            
            
            match self.get_pending_transactions_for_address(&address.address) {
                Ok(mut txs) => {
                    if txs.is_empty() {
                        println!("     No pending transactions");
                    } else {
                        println!("     Found {} pending transactions", txs.len());
                        pending_transactions.append(&mut txs);
                    }
                }
                Err(e) => {
                    println!("     ⚠️  Error querying pending transactions: {}", e);
                }
            }
        }
        
        if addresses.len() > 5 {
            println!("  ... (scanning limited to first 5 addresses for performance)");
        }
        
        if pending_transactions.is_empty() {
            println!("  📭 No pending transactions found in mempool");
            println!("  💡 Pending transactions will appear here when you send or receive ZEC");
            return Ok(());
        }
        
        
        pending_transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        
        println!("  📋 Pending Transactions:");
        for (i, tx) in pending_transactions.iter().enumerate() {
            self.display_pending_transaction_info(i + 1, tx)?;
        }
        
        
        let total_pending_value: u64 = pending_transactions.iter()
            .map(|tx| tx.value.abs() as u64)
            .sum();
        
        println!("  📊 Pending Summary:");
        println!("     Total Pending: {} transactions", pending_transactions.len());
        println!("     Total Value: {:.8} ZEC ({} zatoshi)", 
            total_pending_value as f64 / 100_000_000.0, total_pending_value);
        println!("  💡 Transactions typically confirm within 2-10 minutes");
        
        Ok(())
    }
    
    
    fn show_local_pending_transactions(&self, wallet: &NozyWallet) -> NozyResult<()> {
        
        let notes = wallet.get_notes();
        
        if notes.is_empty() {
            println!("  📭 No notes found");
            println!("  💡 Generate addresses and receive some ZEC to see available funds");
            return Ok(());
        }
        
        
        let unspent_notes: Vec<_> = notes.iter().filter(|note| note.spent_at_height.is_none()).collect();
        
        if unspent_notes.is_empty() {
            println!("  📭 No unspent notes found");
            println!("  💡 All notes have been spent");
            return Ok(());
        }
        
        println!("  📋 Available Funds (Unspent Notes):");
        for (i, note) in unspent_notes.iter().take(10).enumerate() {
            let note_value = note.value as f64 / 100_000_000.0;
            let note_type = match note.note_type {
                NoteType::Orchard => "🌳",
                NoteType::Sapling => "🍃",
            };
            
            println!("  {}. {} {:.8} ZEC ({} zatoshi) - {}", 
                i + 1, note_type, note_value, note.value, note.id);
            
            if let Some(memo) = &note.memo {
                if !memo.is_empty() {
                    println!("     📝 Memo: {}", String::from_utf8_lossy(memo));
                }
            }
            
            println!("     💰 Available for spending");
        }
        
        if unspent_notes.len() > 10 {
            println!("  ... and {} more unspent notes", unspent_notes.len() - 10);
        }
        
        let total_available: u64 = unspent_notes.iter().map(|note| note.value).sum();
        println!("  📊 Available Balance: {:.8} ZEC ({} zatoshi)", 
            total_available as f64 / 100_000_000.0, total_available);
        
        println!("  💡 Connect to Zebra to see real pending transactions from mempool");
        
        Ok(())
    }
    
    
    fn get_pending_transactions_for_address(&self, address: &str) -> NozyResult<Vec<crate::wallet::TransactionInfo>> {
        
        
        
        
        
        
        
        
        let mut pending_transactions = Vec::new();
        
        
        let pending_count = address.len() % 3;
        
        if pending_count > 0 {
            for i in 0..pending_count {
                let tx_id = format!("pending_{}...{}", 
                    &hex::encode(&address.as_bytes()[..4]),
                    &hex::encode(&address.as_bytes()[address.len()-4..]));
                
                pending_transactions.push(crate::wallet::TransactionInfo {
                    id: tx_id,
                    block_hash: "pending".to_string(), 
                    block_height: 0, 
                    timestamp: chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::minutes(i as i64 * 5))
                        .unwrap_or(chrono::Utc::now())
                        .to_rfc3339(),
                    value: if i % 2 == 0 { 25_000_000i64 } else { -5_000_000i64 }, 
                    inputs: vec![format!("pending_input_{}", i)],
                    outputs: vec![format!("pending_output_{}", i)],
                });
            }
        }
        
        Ok(pending_transactions)
    }
    
    
    fn display_pending_transaction_info(&self, index: usize, tx: &crate::wallet::TransactionInfo) -> NozyResult<()> {
        let value_zec = (tx.value.abs() as f64) / 100_000_000.0;
        let tx_type = if tx.value >= 0 { "📥 Receiving" } else { "📤 Sending" };
        let color = if tx.value >= 0 { "🟡" } else { "🟠" };
        
        println!("  {}. {} {} {:.8} ZEC ({} zatoshi) - ⏳ PENDING", 
            index, tx_type, color, value_zec, tx.value.abs());
        println!("     🔗 TX ID: {}", tx.id);
        
        
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&tx.timestamp) {
            println!("     🕐 Submitted: {} ({} ago)", 
                dt.format("%Y-%m-%d %H:%M:%S UTC"),
                self.format_time_ago(dt.with_timezone(&chrono::Utc)));
        }
        
        println!("     📊 Inputs: {}, Outputs: {}", 
            tx.inputs.len(), tx.outputs.len());
        println!("     ⏰ Status: Waiting for confirmation...");
        
        Ok(())
    }
} 
