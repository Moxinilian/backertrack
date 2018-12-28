use super::structure::{Account, Ledger, TransactionMetadata};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

use currency::Currency;

#[allow(dead_code)]
impl Ledger {
    pub fn new_account(&mut self, name: &str, opening_balance: Currency, opening_date: DateTime<Utc>) {
        for v in &self.accounts {
            if v.name == name {
                panic!("An account with name `{}` already exists", name);
            }
        }

        self.accounts.push(Account {
            name: name.to_owned(),
            opening_balance,
            opening_date,
            transactions: Vec::new(),
        });
    }

    pub fn get_account(&self, name: &str) -> Option<&Account> {
        self.accounts.iter().position(|x| x.name == name).and_then(move |x| self.accounts.get(x))
    }

    pub fn get_account_mut(&mut self, name: &str) -> Option<&mut Account> {
        self.accounts.iter().position(|x| x.name == name).and_then(move |x| self.accounts.get_mut(x))
    }

    /*pub fn remove_account(&mut self, name: &str) {
        if let Some(index) = self.accounts.iter().position(|x| x.name == name) {
            self.remove_account_at(index);
        }
    }*/

    pub fn remove_account_at(&mut self, position: usize) {
        self.accounts.remove(position);
    }
}

#[allow(dead_code)]
impl Account {
    pub fn current_balance(&self) -> Currency {
        let mut res = self.opening_balance.clone();

        for t in &self.transactions {
            match t.meta {
                TransactionMetadata::Income { .. } => {
                    res = res + &t.amount;
                }
                TransactionMetadata::Expense { .. } => {
                    res = res - &t.amount;
                }
            }

            for f in &t.fees {
                res = res - &f.amount;
            }
        }

        res
    }

    pub fn sort_by_date(&mut self) {
        self.transactions.sort_by(|x, y| x.date.cmp(&y.date));
    }
}

pub fn new(ledger_path: PathBuf, name: &str, opening_balance: Currency, opening_date: DateTime<Utc>) {
    let mut ledger = Ledger::load(&ledger_path).expect("Could not open the ledger");

    ledger.new_account(name, opening_balance, opening_date);

    ledger
        .save(&ledger_path)
        .expect("Could not save the ledger");
}
