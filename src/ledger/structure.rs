use serde_derive::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use num::BigRational;

use std::error::Error;
use std::path::Path;
use std::fs;

pub type DonationID = Vec<u8>;
pub type PayoutID = Vec<u8>;

#[derive(Serialize, Deserialize)]
pub struct Fee {
    pub towards: String,
    pub amount: BigRational,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum IncomeKind {
    General,
    Donation(DonationID),
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpenseKind {
    General,
    Payout(PayoutID),
}

#[derive(Serialize, Deserialize)]
pub enum TransactionMetadata {
    Income {
        kind: IncomeKind,
        from: String,
    },
    Expense {
        kind: ExpenseKind,
        towards: String,
        requester: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub date: DateTime<Utc>,
    pub description: String,
    pub amount: BigRational,
    pub meta: TransactionMetadata,
    pub fees: Vec<Fee>,
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub opening_date: DateTime<Utc>,
    pub opening_balance: BigRational,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Ledger {
    pub accounts: Vec<Account>,
}

impl Ledger {
    pub fn load(from: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(from)?;
        let ledger = serde_json::from_reader(file)?;
        Ok(ledger)
    }

    pub fn save(&self, to: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let file = fs::File::create(to)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }
}