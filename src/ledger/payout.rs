use std::path::PathBuf;
use serde_derive::Deserialize;
use super::{ExpenseKind, TransactionMetadata, Transaction};

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

pub fn payout(ledger_path: PathBuf, data: PathBuf) {
    let mut ledger = Ledger::load(&ledger_path).expect("Could not read the ledger file");

    match origin {
        PayoutOrigin::PayPal => payout_paypal(&mut ledger, &data),
        PayoutOrigin::Stripe => payout_stripe(&mut ledger, &data),
        PayoutOrigin::Unknown => println!("Unknown origin"),
    }

    ledger
        .save(&ledger_path)
        .expect("Could not save the ledger");
}

#[derive(Deserialize)]
struct StripeRow {
    id: String,
    #[serde(rename = "Amount")]
    amount: f64,
    #[serde(rename = "Created (UTC)")]
    date: String,
}

pub fn payout_stripe(ledger: &mut Ledger, data: &PathBuf) {
    let stripe = ledger
        .get_account_mut("Stripe")
        .expect("Account for Stripe not found");

    let known_payouts: Vec<DonationID> = stripe
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
        let amount = num::BigRational::from_float(x.amount)
            .unwrap_or_else(|| panic!("Could not parse payout amount on entry {}!", i));
        let date = chrono::Utc
            .datetime_from_str(&x.date, "%Y-%m-%d %H:%M")
            .unwrap_or_else(|e| panic!("Could not parse payout date on entry {}!\n{}", i, e));

        let mut hasher = crypto::sha2::Sha256::new();
        hasher.input_str("Stripe");
        hasher.input_str(&x.id);

        let mut hash = vec![0; 32];
        hasher.result(&mut hash);

        if !known_donations.contains(&hash) {
            let meta = TransactionMetadata::Expense {
                kind: ExpenseKind::Payout(hash),
                towards: "Chase".to_owned(),
                requester: "[AUTOMATED]".to_owned(),
            };

            payouts.push(Transaction {
                amount.clone(),
                date.clone(),
                meta,
                description: "Automated payout to the Chase account".to_owned(),
                fees: Vec::new(),
            });

            incomes.push(Transaction {
                amount,
                date,
                description: "Automated payout from Stripe".to_owned(),
                fees: Vec::new(),
                meta: TransactionMetadata::Income {
                    kind: IncomeKind::General,
                    from: "Stripe payout".to_owned(),
                }
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