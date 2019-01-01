use super::{Account, Ledger, TransactionMetadata};
use std::path::PathBuf;
use currency::Currency;

pub fn info(ledger: PathBuf, accounts: &str) {
    let ledger = Ledger::load(&ledger).expect("Could not read the ledger file");

    let accounts: Vec<&Account> = accounts
        .split(',')
        .map(|x| {
            ledger
                .get_account(x)
                .unwrap_or_else(|| panic!("Account `{}` not found in the ledger", x))
        })
        .collect();

    let gross_receipts = accounts
        .iter()
        .map(|x| &x.transactions)
        .flatten()
        .filter(|x| match x.meta {
            TransactionMetadata::Income { .. } => true,
            _ => false,
        })
        .map(|x| &x.amount)
        .fold(Currency::from(0, '$'), |acc, x| x + acc);

    println!("Gross receipts: {}", gross_receipts);
}
