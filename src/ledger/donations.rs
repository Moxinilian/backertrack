use super::{DonationID, Fee, IncomeKind, Ledger, Transaction, TransactionMetadata};
use chrono::offset::TimeZone;
use crypto::digest::Digest;
use serde_derive::Deserialize;
use std::path::PathBuf;

pub enum DonationOrigin {
    Unknown,
    DonorBox,
    OpenCollective,
}

impl From<&str> for DonationOrigin {
    fn from(from: &str) -> Self {
        match from {
            "donorbox" => DonationOrigin::DonorBox,
            "opencollective" => DonationOrigin::OpenCollective,
            _ => DonationOrigin::Unknown,
        }
    }
}

pub fn import(ledger_path: PathBuf, data: PathBuf, origin: DonationOrigin) {
    let mut ledger = Ledger::load(&ledger_path).expect("Could not read the ledger file");

    match origin {
        DonationOrigin::DonorBox => import_donorbox(&mut ledger, &data),
        DonationOrigin::OpenCollective => import_opencollective(&mut ledger, &data),
        DonationOrigin::Unknown => println!("Unknown origin"),
    }

    ledger.sort_by_date();
    ledger
        .save(&ledger_path)
        .expect("Could not save the ledger");
}

#[derive(Deserialize)]
struct OpenCollectiveRow {
    #[serde(rename = "User Name")]
    user: String,
    #[serde(rename = "Transaction Date")]
    date: String,
    #[serde(rename = "Transaction Amount")]
    amount: f64,
    #[serde(rename = "Host Fee (USD)")]
    host_fee: f64,
    #[serde(rename = "Open Collective Fee (USD)")]
    oc_fee: f64,
    #[serde(rename = "Payment Processor Fee (USD)")]
    processor_fee: f64,
    #[serde(rename = "Net Amount (USD)")]
    net_amount: String,
}

fn import_opencollective(ledger: &mut Ledger, data: &PathBuf) {
    let account = ledger
        .get_account_mut("Stripe")
        .expect("Account for Stripe not found");

    let known_donations: Vec<DonationID> = account
        .transactions
        .iter()
        .filter_map(|x| match &x.meta {
            TransactionMetadata::Income {
                kind: IncomeKind::Donation(x),
                ..
            } => Some(x),
            _ => None,
        })
        .map(Clone::clone)
        .collect();

    let mut donations: Vec<Transaction> = Vec::new();

    let mut reader = csv::Reader::from_path(data).expect("Could not read the CSV file");

    let records = reader.deserialize();
    for (i, x) in records.enumerate() {
        let x: OpenCollectiveRow =
            x.unwrap_or_else(|e| panic!("Could not deserialize entry on entry {}!\n{}", i, e));
        let amount = num::BigRational::from_float(x.amount)
            .unwrap_or_else(|| panic!("Could not parse transaction amount on entry {}!", i));
        let host_fee = num::BigRational::from_float(x.host_fee)
            .unwrap_or_else(|| panic!("Could not parse host fee on entry {}!", i));
        let oc_fee = num::BigRational::from_float(x.oc_fee)
            .unwrap_or_else(|| panic!("Could not parse OpenCollective fee on entry {}!", i));
        let processor_fee = num::BigRational::from_float(x.processor_fee)
            .unwrap_or_else(|| panic!("Could not parse processor fee fee on entry {}!", i));
        let date = chrono::Utc
            .datetime_from_str(&x.date, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|e| panic!("Could not parse transaction date on entry {}!\n{}", i, e));

        let mut hasher = crypto::sha2::Sha256::new();
        hasher.input_str("OpenCollective");
        hasher.input_str(&x.user);
        hasher.input_str(&x.date);
        hasher.input_str(&x.net_amount);

        let mut hash = vec![0; 32];
        hasher.result(&mut hash);

        if !known_donations.contains(&hash) {
            let meta = TransactionMetadata::Income {
                kind: IncomeKind::Donation(hash),
                from: x.user.to_owned(),
            };

            //println!("Processor: {}\nOC: {}\nHost: {}\n\n", processor_fee, oc_fee, host_fee);

            donations.push(Transaction {
                amount,
                date,
                meta,
                description: "Donation made through the OpenCollective platform".to_owned(),
                fees: vec![
                    Fee {
                        amount: host_fee.clone(),
                        towards: "Collective Host (Amethyst Foundation)".to_owned(),
                    },
                    Fee {
                        amount: -host_fee,
                        towards: "Collective Host (Amethyst Foundation)".to_owned(),
                    },
                    Fee {
                        amount: -oc_fee,
                        towards: "OpenCollective".to_owned(),
                    },
                    Fee {
                        amount: -processor_fee,
                        towards: "Payment Processor".to_owned(),
                    },
                ],
            });
        } else {
            println!(
                "WARNING: Donation from `{}` on {} (entry {}) is already in the ledger.",
                x.user, x.date, i
            );
        }
    }

    donations.sort_by(|x, y| x.date.cmp(&y.date));
    account.transactions.append(&mut donations);
}

#[derive(Deserialize)]
struct DonorBoxRow {
    #[serde(rename = "Date Donated")]
    date: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Amount")]
    amount: f64,
    #[serde(rename = "Processing Fee")]
    fee: f64,
    #[serde(rename = "Net Amount")]
    net_amount: String,
    #[serde(rename = "Receipt Id")]
    receipt: String,
    #[serde(rename = "Donation Type")]
    processor: String,
}

fn import_donorbox(ledger: &mut Ledger, data: &PathBuf) {
    let stripe = ledger
        .get_account("Stripe")
        .expect("Account for Stripe not found");

    let paypal = ledger
        .get_account("PayPal")
        .expect("Account for PayPal not found");

    let known_donations: Vec<DonationID> = stripe
        .transactions
        .iter()
        .chain(&paypal.transactions)
        .filter_map(|x| match &x.meta {
            TransactionMetadata::Income {
                kind: IncomeKind::Donation(x),
                ..
            } => Some(x),
            _ => None,
        })
        .map(Clone::clone)
        .collect();

    let mut donations_stripe: Vec<Transaction> = Vec::new();
    let mut donations_paypal: Vec<Transaction> = Vec::new();

    let mut reader = csv::Reader::from_path(data).expect("Could not read the CSV file");

    let records = reader.deserialize();
    for (i, x) in records.enumerate() {
        let x: DonorBoxRow =
            x.unwrap_or_else(|e| panic!("Could not deserialize entry on entry {}!\n{}", i, e));
        let amount = num::BigRational::from_float(x.amount)
            .unwrap_or_else(|| panic!("Could not parse transaction amount on entry {}!", i));
        let fee = num::BigRational::from_float(x.fee)
            .unwrap_or_else(|| panic!("Could not parse host fee on entry {}!", i));
        let date = chrono::Utc
            .datetime_from_str(&x.date.trim_end_matches(" UTC"), "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|e| panic!("Could not parse transaction date on entry {}!\n{}", i, e));

        let mut hasher = crypto::sha2::Sha256::new();
        hasher.input_str("DonorBox");
        hasher.input_str(&x.name);
        hasher.input_str(&x.date);
        hasher.input_str(&x.net_amount);
        hasher.input_str(&x.receipt);

        let mut hash = vec![0; 32];
        hasher.result(&mut hash);

        if !known_donations.contains(&hash) {
            let meta = TransactionMetadata::Income {
                kind: IncomeKind::Donation(hash),
                from: x.name.to_owned(),
            };

            match x.processor.as_ref() {
                "stripe" => {
                    donations_stripe.push(Transaction {
                        amount,
                        date,
                        meta,
                        description: "Donation made through the DonorBox platform".to_owned(),
                        fees: vec![Fee {
                            amount: fee,
                            towards: "DonorBox Processing".to_owned(),
                        }],
                    });
                }
                "paypal" | "paypal_express" => {
                    donations_paypal.push(Transaction {
                        amount,
                        date,
                        meta,
                        description: "Donation made through the DonorBox platform".to_owned(),
                        fees: vec![Fee {
                            amount: fee,
                            towards: "DonorBox Processing".to_owned(),
                        }],
                    });
                }
                mtd => println!("WARNING: Unknown donation method `{}` for donation from `{}` on {} (entry {}).", mtd, x.name, x.date, i),
            }
        } else {
            println!(
                "WARNING: Donation from `{}` on {} (entry {}) is already in the ledger.",
                x.name, x.date, i
            );
        }
    }

    donations_stripe.sort_by(|x, y| x.date.cmp(&y.date));
    ledger
        .get_account_mut("Stripe")
        .expect("Account for Stripe not found")
        .transactions
        .append(&mut donations_stripe);

    donations_paypal.sort_by(|x, y| x.date.cmp(&y.date));
    ledger
        .get_account_mut("PayPal")
        .expect("Account for PayPal not found")
        .transactions
        .append(&mut donations_paypal);
}
