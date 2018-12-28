use super::{ExpenseKind, IncomeKind, Ledger, TransactionMetadata, Fee};
use serde_derive::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct ExportRow<'a> {
    account: &'a str,
    kind: &'a str,
    date: &'a str,
    amount: &'a str,
    fees: &'a str,
    description: &'a str,
    paid_to: &'a str,
    paid_by: &'a str,
}

pub fn export(ledger: PathBuf, to: PathBuf) {
    let ledger = Ledger::load(&ledger).expect("Could not read the ledger file");
    let mut writer = csv::Writer::from_path(to).expect("Could not open the target file");

    for account in ledger.accounts {
        writer.serialize(ExportRow {
            account: &account.name,
            kind: "Opening",
            amount: &(-account.opening_balance).to_string(),
            date: &account.opening_date.to_rfc3339(),
            fees: "",
            description: "",
            paid_to: "",
            paid_by: "",
        }).expect("Failed to serialize opening");
        for transaction in account.transactions {
            match transaction.meta {
                TransactionMetadata::Expense {
                    kind: ExpenseKind::General,
                    ref towards,
                    ..
                } => {
                    writer.serialize(ExportRow {
                        account: &account.name,
                        kind: "Expense",
                        amount: &transaction.amount.to_string(),
                        date: &transaction.date.to_rfc3339(),
                        fees: &format_fees(&transaction.fees),
                        description: &transaction.description,
                        paid_to: towards,
                        paid_by: "",
                    }).expect("Failed to serialize general expense");
                }
                TransactionMetadata::Expense {
                    kind: ExpenseKind::Payout(_),
                    ref towards,
                    ..
                } => {
                    writer.serialize(ExportRow {
                        account: &account.name,
                        kind: "Payout",
                        amount: &transaction.amount.to_string(),
                        date: &transaction.date.to_rfc3339(),
                        fees: &format_fees(&transaction.fees),
                        description: &transaction.description,
                        paid_to: towards,
                        paid_by: "",
                    }).expect("Failed to serialize payout expense");
                }
                TransactionMetadata::Income {
                    kind: IncomeKind::General,
                    ref from,
                } => {
                    writer.serialize(ExportRow {
                        account: &account.name,
                        kind: "Income",
                        amount: &(-transaction.amount).to_string(),
                        date: &transaction.date.to_rfc3339(),
                        fees: &format_fees(&transaction.fees),
                        description: &transaction.description,
                        paid_to: "",
                        paid_by: from,
                    }).expect("Failed to serialize general income");
                }
                TransactionMetadata::Income {
                    kind: IncomeKind::Donation(_),
                    ref from,
                } => {
                    writer.serialize(ExportRow {
                        account: &account.name,
                        kind: "Donation",
                        amount: &(-transaction.amount).to_string(),
                        date: &transaction.date.to_rfc3339(),
                        fees: &format_fees(&transaction.fees),
                        description: &transaction.description,
                        paid_to: "",
                        paid_by: from,
                    }).expect("Failed to serialize donation");
                }
            }
        }
    }
}

fn format_fees(fees: &Vec<Fee>) -> String {
    let mut res = String::new();
    for f in fees {
        res.push_str(&format!("{}[{}];", &f.amount.to_string(), f.towards));
    }
    res.pop();
    res
}