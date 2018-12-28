use super::{
    super::tui_utils::Event,
    text::{generate_help_text, generate_info_text, generate_input_fields_text},
    utils::{decrease_modular, increase_modular},
    LedgerTab, LedgerTabState, NewTransactionKind, Trans,
};
use crate::ledger;
use crate::utils::GetOrDefault;
use num::traits::FromPrimitive;
use termion::event::Key;

use lazy_static::lazy_static;
use regex::Regex;
lazy_static! {
    static ref FEES_REGEX: Regex = { Regex::new(r"([^\[]+)\[([^\]]+)\]").unwrap() };
}

pub const FIELDS: &'static [&'static str] = &["Kind"];
pub const FIELDS_KIND: &'static [&'static [&'static str]] = &[
    &[
        "UTC date (YYYY/MM/DD HH:MM)",
        "Description",
        "Amount",
        "Fees",
        "From",
    ],
    &[
        "UTC date (YYYY/MM/DD HH:MM)",
        "Description",
        "Amount",
        "Fees",
        "Towards",
        "Requester",
    ],
    &[
        "UTC date (YYYY/MM/DD HH:MM)",
        "Description",
        "Amount",
        "Fees",
        "From",
        "Donation ID",
    ],
    &[
        "UTC date (YYYY/MM/DD HH:MM)",
        "Description",
        "Amount",
        "Fees",
        "Towards",
        "Requester",
        "Payout ID",
    ],
];

impl NewTransactionKind {
    pub fn get_name(self) -> &'static str {
        match self {
            NewTransactionKind::DonationIncome => "Donation",
            NewTransactionKind::GeneralExpense => "Expense",
            NewTransactionKind::GeneralIncome => "Income",
            NewTransactionKind::PayoutExpense => "Payout Expense",
        }
    }
}

pub fn event(tab: &mut LedgerTab, event: Event<Key>) -> Trans {
    if let LedgerTabState::NewTransaction(ref mut state, ref mut selected) = tab.state {
        match state {
            0 => match event {
                Event::Input(Key::Esc) => {
                    tab.text_input_fields.clear();
                    tab.selected_field = 0;
                    tab.state = LedgerTabState::Normal;
                }
                Event::Input(Key::Up) => {
                    decrease_modular(&mut tab.selected_field, 1, FIELDS.len() + 1);
                    generate_input_fields_text(tab, FIELDS, "Next");
                }
                Event::Input(Key::Down) => {
                    increase_modular(&mut tab.selected_field, 1, FIELDS.len() + 1);
                    generate_input_fields_text(tab, FIELDS, "Next");
                }
                Event::Input(Key::Left) => {
                    let mut selected_int = *selected as usize;
                    decrease_modular(&mut selected_int, 1, FIELDS_KIND.len());
                    *selected = NewTransactionKind::from_usize(selected_int)
                        .expect("Unreachable: from_usize left");
                    *tab.text_input_fields.get_mut_or_default(0) = selected.get_name().to_owned();
                    generate_input_fields_text(tab, FIELDS, "Next");
                }
                Event::Input(Key::Right) => {
                    let mut selected_int = *selected as usize;
                    increase_modular(&mut selected_int, 1, FIELDS_KIND.len());
                    *selected = NewTransactionKind::from_usize(selected_int)
                        .expect("Unreachable: from_usize right");
                    *tab.text_input_fields.get_mut_or_default(0) = selected.get_name().to_owned();
                    generate_input_fields_text(tab, FIELDS, "Next");
                }
                Event::Input(Key::Char('\n')) => {
                    if tab.selected_field == FIELDS.len() {
                        tab.text_input_fields.clear();
                        let fields = FIELDS_KIND[*selected as usize];
                        tab.selected_field = 0;
                        *state = 1;
                        generate_input_fields_text(tab, fields, "Confirm");
                    }
                }
                _ => {}
            },
            1 => match event {
                Event::Input(Key::Esc) => {
                    tab.text_input_fields.clear();
                    tab.selected_field = 0;
                    tab.state = LedgerTabState::Normal;
                }
                Event::Input(Key::Up) => {
                    let fields = FIELDS_KIND[*selected as usize];
                    decrease_modular(&mut tab.selected_field, 1, fields.len() + 1);
                    generate_input_fields_text(tab, fields, "Confirm");
                }
                Event::Input(Key::Down) => {
                    let fields = FIELDS_KIND[*selected as usize];
                    increase_modular(&mut tab.selected_field, 1, fields.len() + 1);
                    generate_input_fields_text(tab, fields, "Confirm");
                }
                Event::Input(Key::Char('\n')) => {
                    if tab.selected_field == FIELDS_KIND[*selected as usize].len() {
                        use chrono::offset::TimeZone;
                        if let Ok(date) = chrono::Utc.datetime_from_str(
                            &tab.text_input_fields.get_or_default(0),
                            crate::DATE_FORMAT,
                        ) {
                            if let Ok(amount) = tab
                                .text_input_fields
                                .get(2)
                                .expect("Unreachable: new_txn amount")
                                .parse::<f64>()
                            {
                                if let Some(amount) = num::BigRational::from_float(amount) {
                                    if let Some(fees) = parse_fees(
                                        tab.text_input_fields
                                            .get(3)
                                            .expect("Unreachable: new_txn fees"),
                                    ) {
                                        if let Some(meta) = match selected {
                                            NewTransactionKind::GeneralExpense => {
                                                Some(ledger::TransactionMetadata::Expense {
                                                    kind: ledger::ExpenseKind::General,
                                                    towards: tab
                                                        .text_input_fields
                                                        .get(4)
                                                        .expect("Unreachable: new_txn towards")
                                                        .to_owned(),
                                                    requester: tab
                                                        .text_input_fields
                                                        .get(5)
                                                        .expect("Unreachable: new_txn requester")
                                                        .to_owned(),
                                                })
                                            }
                                            NewTransactionKind::GeneralIncome => {
                                                Some(ledger::TransactionMetadata::Income {
                                                    kind: ledger::IncomeKind::General,
                                                    from: tab
                                                        .text_input_fields
                                                        .get(4)
                                                        .expect("Unreachable: new_txn from general")
                                                        .to_owned(),
                                                })
                                            }
                                            NewTransactionKind::DonationIncome => hex::decode(
                                                tab.text_input_fields
                                                    .get(5)
                                                    .expect("Unreachable: new_txn donation id"),
                                            )
                                            .ok()
                                            .map(|x| ledger::TransactionMetadata::Income {
                                                kind: ledger::IncomeKind::Donation(x),
                                                from: tab
                                                    .text_input_fields
                                                    .get(4)
                                                    .expect("Unreachable: new_txn from donation")
                                                    .to_owned(),
                                            }),
                                            NewTransactionKind::PayoutExpense => hex::decode(
                                                tab.text_input_fields
                                                    .get(6)
                                                    .expect("Unreachable: new_txn payout id"),
                                            )
                                            .ok()
                                            .map(|x| ledger::TransactionMetadata::Expense {
                                                    kind: ledger::ExpenseKind::Payout(x),
                                                    towards: tab
                                                        .text_input_fields
                                                        .get(4)
                                                        .expect("Unreachable: new_txn towards payout")
                                                        .to_owned(),
                                                    requester: tab
                                                        .text_input_fields
                                                        .get(5)
                                                        .expect("Unreachable: new_txn requester payout")
                                                        .to_owned(),
                                                })
                                        } {
                                            let account = tab
                                                .ledger
                                                .accounts
                                                .get_mut(tab.account_cursor)
                                                .expect("Unreachable: new_txn acc cursor bounds");
                                            account.transactions.push(ledger::Transaction {
                                                amount,
                                                date,
                                                description: tab
                                                    .text_input_fields
                                                    .get(1)
                                                    .expect("Unreachable: new_txn description")
                                                    .to_owned(),
                                                meta,
                                                fees,
                                            });
                                            tab.text_input_fields.clear();
                                            tab.selected_field = 0;
                                            tab.state = LedgerTabState::Normal;
                                            tab.transactions_names =
                                                super::text::generate_transaction_names(
                                                    &tab.ledger,
                                                );
                                            generate_info_text(tab);
                                            generate_help_text(tab);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Event::Input(Key::Char(x)) => {
                    if let Some(field) = tab.text_input_fields.get_mut(tab.selected_field) {
                        field.push(x);
                        let fields = FIELDS_KIND[*selected as usize];
                        generate_input_fields_text(tab, fields, "Confirm");
                    }
                }
                Event::Input(Key::Backspace) => {
                    if let Some(field) = tab.text_input_fields.get_mut(tab.selected_field) {
                        field.pop();
                        let fields = FIELDS_KIND[*selected as usize];
                        generate_input_fields_text(tab, fields, "Confirm");
                    }
                }
                _ => {}
            },
            x => panic!("Unreachable: invalid new_txn state {}", x),
        }
    }
    Trans::None
}

fn parse_fees(fees_str: &str) -> Option<Vec<ledger::Fee>> {
    let mut fees = Vec::new();

    for x in fees_str.split(';') {
        if x.trim() != "" {
            if let Some(c) = FEES_REGEX.captures(x) {
                if let Some(amount) = c.get(1) {
                    if let Ok(amount) = amount.as_str().parse::<f64>() {
                        if let Some(amount) = num::BigRational::from_float(amount) {
                            if let Some(towards) = c.get(2) {
                                fees.push(ledger::Fee {
                                    amount,
                                    towards: towards.as_str().to_owned(),
                                });
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }

    Some(fees)
}
