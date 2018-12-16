use super::{
    super::tui_utils::Event,
    text::{generate_help_text, generate_info_text, generate_input_fields_text},
    utils::{decrease_modular, increase_modular},
    LedgerList, LedgerTab, LedgerTabState, MainTab, OrdinaryFrame, Trans,
};
use crate::utils::GetOrDefault;
use crate::{
    ledger::{ExpenseKind, IncomeKind, Ledger, TransactionMetadata},
    utils::display_currency,
};
use std::borrow::Cow;
use termion::event::Key;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Paragraph, SelectableList, Tabs, Text, Widget},
};

pub fn event(tab: &mut LedgerTab, event: Event<Key>) -> Trans {
    match event {
        Event::Input(Key::Ctrl('Q')) => Trans::Quit,
        Event::Input(Key::Ctrl('S')) => {
            tab.ledger
                .save(&tab.ledger_path)
                .unwrap_or_else(|e| panic!("Could not save ledger!\n{}", e));
            Trans::None
        }
        Event::Input(Key::Esc) => {
            tab.ledger
                .save(&tab.ledger_path)
                .unwrap_or_else(|e| panic!("Could not save ledger!\n{}", e));
            Trans::Quit
        }
        Event::Input(Key::Up) => match tab.active_list {
            LedgerList::Accounts => {
                decrease_modular(&mut tab.account_cursor, 1, tab.accounts_names.len());
                generate_info_text(tab);
                Trans::None
            }
            LedgerList::Transactions => {
                let transactions_amount = tab
                    .ledger
                    .accounts
                    .get(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds in ledger")
                    .transactions
                    .len();
                let cursor = tab
                    .accounts_cursors
                    .get_mut(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds");
                decrease_modular(cursor, 1, transactions_amount);
                generate_info_text(tab);
                Trans::None
            }
        },
        Event::Input(Key::Down) => match tab.active_list {
            LedgerList::Accounts => {
                increase_modular(&mut tab.account_cursor, 1, tab.accounts_names.len());
                generate_info_text(tab);
                Trans::None
            }
            LedgerList::Transactions => {
                let transactions_amount = tab
                    .ledger
                    .accounts
                    .get(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds in ledger")
                    .transactions
                    .len();
                let cursor = tab
                    .accounts_cursors
                    .get_mut(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds");
                increase_modular(cursor, 1, transactions_amount);
                generate_info_text(tab);
                Trans::None
            }
        },
        Event::Input(Key::PageUp) => match tab.active_list {
            LedgerList::Accounts => {
                decrease_modular(&mut tab.account_cursor, 5, tab.accounts_names.len());
                generate_info_text(tab);
                Trans::None
            }
            LedgerList::Transactions => {
                let transactions_amount = tab
                    .ledger
                    .accounts
                    .get(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds in ledger")
                    .transactions
                    .len();
                let cursor = tab
                    .accounts_cursors
                    .get_mut(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds");
                decrease_modular(cursor, 5, transactions_amount);
                generate_info_text(tab);
                Trans::None
            }
        },
        Event::Input(Key::PageDown) => match tab.active_list {
            LedgerList::Accounts => {
                increase_modular(&mut tab.account_cursor, 5, tab.accounts_names.len());
                generate_info_text(tab);
                Trans::None
            }
            LedgerList::Transactions => {
                let transactions_amount = tab
                    .ledger
                    .accounts
                    .get(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds in ledger")
                    .transactions
                    .len();
                let cursor = tab
                    .accounts_cursors
                    .get_mut(tab.account_cursor)
                    .expect("Unreachable: account cursor out of bounds");
                increase_modular(cursor, 5, transactions_amount);
                generate_info_text(tab);
                Trans::None
            }
        },
        Event::Input(Key::Right) | Event::Input(Key::Left) => {
            match tab.active_list {
                LedgerList::Accounts => {
                    if let Some(account) = tab.ledger.accounts.get(tab.account_cursor) {
                        if account.transactions.len() != 0 {
                            tab.active_list = LedgerList::Transactions;
                            generate_help_text(tab);
                            generate_info_text(tab);
                        }
                    }
                }
                LedgerList::Transactions => {
                    tab.active_list = LedgerList::Accounts;
                    generate_help_text(tab);
                    generate_info_text(tab);
                }
            }
            Trans::None
        }
        Event::Input(Key::Char('+')) => {
            tab.state = LedgerTabState::NewAccount;
            generate_input_fields_text(tab, super::new_account::FIELDS, "Confirm");
            Trans::None
        }
        Event::Input(Key::Char('\n')) => {
            if tab.accounts_names.len() != 0 {
                tab.state =
                    LedgerTabState::NewTransaction(0, super::NewTransactionKind::GeneralExpense);
                *tab.text_input_fields.get_mut_or_default(0) =
                    super::NewTransactionKind::GeneralExpense
                        .get_name()
                        .to_owned();
                generate_input_fields_text(tab, super::new_txn::FIELDS, "Next");
            }
            Trans::None
        }
        Event::Input(Key::Char('-')) => {
            if tab.accounts_names.len() != 0 {
                tab.state = LedgerTabState::Delete;
            }
            Trans::None
        }
        Event::Input(Key::Char('n')) => {
            let name = format!(
                "{}a",
                tab.ledger
                    .accounts
                    .last()
                    .map(|x| &x.name)
                    .unwrap_or(&"".to_owned())
            );
            tab.ledger
                .new_account(&name, num::BigRational::from_integer(num::BigInt::from(0)), chrono::Utc::now());
            tab.accounts_cursors.push(0);
            tab.transactions_names.push(Vec::new());
            tab.accounts_names.push(name);
            generate_info_text(tab);
            Trans::None
        }
        Event::Input(Key::Char('t')) => {
            let txn = crate::ledger::Transaction {
                date: chrono::Utc::now(),
                description: "Acquire the best mobile office".to_owned(),
                amount: num::BigRational::from_integer(num::BigInt::from(10000001))
                    / num::BigRational::from_integer(num::BigInt::from(3)),
                fees: Vec::new(),
                meta: TransactionMetadata::Expense {
                    kind: ExpenseKind::General,
                    towards: "HighestQualityYachtsInc".to_owned(),
                    requester: "Moxinilian".to_owned(),
                },
            };
            tab.ledger
                .accounts
                .get_mut(tab.account_cursor)
                .unwrap()
                .transactions
                .push(txn);
            tab.transactions_names = super::text::generate_transaction_names(&tab.ledger);
            generate_info_text(tab);
            Trans::None
        }

        _ => Trans::None,
    }
}
