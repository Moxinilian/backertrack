use super::{ExpenseKind, IncomeKind, Ledger, PayoutID, Transaction, TransactionMetadata};
use chrono::TimeZone;
use crypto::digest::Digest;
use serde_derive::Deserialize;
use std::path::PathBuf;

pub enum PayoutOrigin {
    Unknown,
    PayPal,
    Stripe,
}

impl From<&str> for PayoutOrigin {
    fn from(from: &str) -> Self {
        match from {
            "paypal" => PayoutOrigin::PayPal,
            "stripe" => PayoutOrigin::Stripe,
            _ => PayoutOrigin::Unknown,
        }
    }
}

pub fn payout(ledger_path: PathBuf, data: PathBuf, origin: PayoutOrigin) {
    let mut ledger = Ledger::load(&ledger_path).expect("Could not read the ledger file");

    match origin {
        PayoutOrigin::PayPal => payout_paypal(&mut ledger, &data),
        PayoutOrigin::Stripe => payout_stripe(&mut ledger, &data),
        PayoutOrigin::Unknown => println!("Unknown origin"),
    }

    ledger.sort_by_date();
    ledger
        .save(&ledger_path)
        .expect("Could not save the ledger");
}

#[derive(Deserialize)]
struct StripeRow {
    id: String,
    #[serde(rename = "Amount")]
    amount: String,
    #[serde(rename = "Created (UTC)")]
    date: String,
}

fn payout_stripe(ledger: &mut Ledger, data: &PathBuf) {
    let stripe = ledger
        .get_account_mut("Stripe")
        .expect("Account for Stripe not found");

    let known_payouts: Vec<PayoutID> = stripe
        .transactions
        .iter()
        .filter_map(|x| match &x.meta {
            TransactionMetadata::Expense {
                kind: ExpenseKind::Payout(x),
                ..
            } => Some(x),
            _ => None,
        })
        .map(Clone::clone)
        .collect();

    let mut payouts: Vec<Transaction> = Vec::new();
    let mut incomes: Vec<Transaction> = Vec::new();

    let mut reader = csv::Reader::from_path(data).expect("Could not read the CSV file");

    let records = reader.deserialize();
    for (i, x) in records.enumerate() {
        let x: StripeRow =
            x.unwrap_or_else(|e| panic!("Could not deserialize entry on entry {}!\n{}", i, e));
        let mut amount = currency::Currency::from_str(&x.amount)
            .unwrap_or_else(|e| panic!("Could not parse payout amount on entry {}!\n{}", i, e));
        let date = chrono::Utc
            .datetime_from_str(&x.date, "%Y-%m-%d %H:%M")
            .unwrap_or_else(|e| {
                panic!(
                    "Could not parse payout date `{}` on entry {}!\n{}",
                    &x.date, i, e
                )
            });

        amount.set_symbol('$');

        let mut hasher = crypto::sha2::Sha256::new();
        hasher.input_str("Stripe");
        hasher.input_str(&x.id);

        let mut hash = vec![0; 32];
        hasher.result(&mut hash);

        if !known_payouts.contains(&hash) {
            let meta = TransactionMetadata::Expense {
                kind: ExpenseKind::Payout(hash),
                towards: "Chase".to_owned(),
                requester: "Treasurer".to_owned(),
            };

            payouts.push(Transaction {
                amount: amount.clone(),
                date: date.clone(),
                meta,
                description: "Payout to the Chase account".to_owned(),
                fees: Vec::new(),
            });

            incomes.push(Transaction {
                amount,
                date,
                description: "Payout from Stripe".to_owned(),
                fees: Vec::new(),
                meta: TransactionMetadata::Income {
                    kind: IncomeKind::General,
                    from: "Stripe payout".to_owned(),
                },
            })
        } else {
            println!(
                "WARNING: Payout from made on {} (entry {}) is already in the ledger.",
                x.date, i
            );
        }
    }

    payouts.sort_by(|x, y| x.date.cmp(&y.date));
    stripe.transactions.append(&mut payouts);

    incomes.sort_by(|x, y| x.date.cmp(&y.date));
    ledger
        .get_account_mut("Chase")
        .expect("Account for Chase not found")
        .transactions
        .append(&mut incomes);
}

#[derive(Deserialize)]
struct PayPalRow {
    #[serde(rename = "Transaction ID")]
    id: String,
    #[serde(rename = "Gross")]
    amount: String,
    #[serde(rename = "Date")]
    date: String,
}

fn payout_paypal(ledger: &mut Ledger, data: &PathBuf) {
    let paypal = ledger
        .get_account_mut("PayPal")
        .expect("Account for PayPal not found");

    let known_payouts: Vec<PayoutID> = paypal
        .transactions
        .iter()
        .filter_map(|x| match &x.meta {
            TransactionMetadata::Expense {
                kind: ExpenseKind::Payout(x),
                ..
            } => Some(x),
            _ => None,
        })
        .map(Clone::clone)
        .collect();

    let mut payouts: Vec<Transaction> = Vec::new();
    let mut incomes: Vec<Transaction> = Vec::new();

    let mut reader = csv::Reader::from_path(data).expect("Could not read the CSV file");

    let records = reader.deserialize();
    for (i, x) in records.enumerate() {
        let x: PayPalRow =
            x.unwrap_or_else(|e| panic!("Could not deserialize entry on entry {}!\nHave you done the necessary preprocessing for PayPal exported data?\n{}", i, e));
        let mut amount = currency::Currency::from_str(&x.amount)
            .unwrap_or_else(|e| panic!("Could not parse payout amount on entry {}!\n{}", i, e));
        let date = chrono::Utc
            .datetime_from_str(&format!("{} 00:00", &x.date), "%m/%d/%Y %H:%M")
            .unwrap_or_else(|e| {
                panic!(
                    "Could not parse payout date `{}` on entry {}!\n{}",
                    &x.date, i, e
                )
            });

        amount.set_symbol('$');

        let mut hasher = crypto::sha2::Sha256::new();
        hasher.input_str("PayPal");
        hasher.input_str(&x.id);

        let mut hash = vec![0; 32];
        hasher.result(&mut hash);

        if !known_payouts.contains(&hash) {
            let amount = -amount;

            let meta = TransactionMetadata::Expense {
                kind: ExpenseKind::Payout(hash),
                towards: "Chase".to_owned(),
                requester: "Treasurer".to_owned(),
            };

            payouts.push(Transaction {
                amount: amount.clone(),
                date: date.clone(),
                meta,
                description: "Payout to the Chase account".to_owned(),
                fees: Vec::new(),
            });

            incomes.push(Transaction {
                amount,
                date,
                description: "Payout from PayPal".to_owned(),
                fees: Vec::new(),
                meta: TransactionMetadata::Income {
                    kind: IncomeKind::General,
                    from: "PayPal payout".to_owned(),
                },
            })
        } else {
            println!(
                "WARNING: Payout from made on {} (entry {}) is already in the ledger.",
                x.date, i
            );
        }
    }

    payouts.sort_by(|x, y| x.date.cmp(&y.date));
    paypal.transactions.append(&mut payouts);

    incomes.sort_by(|x, y| x.date.cmp(&y.date));
    ledger
        .get_account_mut("Chase")
        .expect("Account for Chase not found")
        .transactions
        .append(&mut incomes);
}
