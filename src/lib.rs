
// Nozy the wallet for privacy advocates.

pub mod error;
pub mod config;
pub mod storage;
pub mod notes;
pub mod addresses;
pub mod transactions;
pub mod zebra_integration;
pub mod hd_wallet;
pub mod encrypted_storage;
pub mod transaction_signer;
pub mod wallet;
pub mod cli;

pub use error::{NozyError, NozyResult};
pub use config::{NozyConfig, PrivacyLevel};
pub use storage::WalletStorage;
pub use notes::{NoteManager, ShieldedNote, NoteType};
pub use addresses::{AddressManager, ZcashAddressWrapper, ZcashAddressType};
pub use transactions::{TransactionBuilder, ShieldedTransaction, TransactionInput, TransactionOutput, TransactionStatus};
pub use zebra_integration::{ZebraClient, ZebraConfig, ZebraStatus, SyncStatus};
pub use wallet::{NozyWallet, WalletStatus};
pub use hd_wallet::{HDWallet, AddressType};
pub use encrypted_storage::EncryptedStorage;
pub use transaction_signer::{TransactionSigner, ShieldedInput, ShieldedOutput, SignedTransaction};
pub use cli::{Cli, CliHandler, Commands};

/// Main entry point for the Nozy wallet so dont be Nozy noting to see here
pub fn hello_nozy() -> &'static str {
    "Hello from Nozy wallet!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello_nozy(), "Hello from Nozy wallet!");
    }
} 