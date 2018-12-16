mod structure;
pub use self::structure::*;

pub mod accounts;
pub mod donations;

mod export;
pub use self::export::export;

use std::path::PathBuf;

pub fn new(ledger: PathBuf) {
    self::structure::Ledger::default().save(&ledger).expect("Failed to save the ledger!");
}

