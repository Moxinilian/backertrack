mod structure;
pub use self::structure::*;

pub mod accounts;
pub mod donations;
pub mod payout;

mod export;
pub use self::export::export;

mod info;
pub use self::info::info;

use std::path::PathBuf;

pub fn new(ledger: PathBuf) {
    self::structure::Ledger::default().save(&ledger).expect("Failed to save the ledger!");
}

