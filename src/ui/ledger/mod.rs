use super::{tui_utils::Event, MainTab, OrdinaryFrame, Trans};
use crate::{
    ledger::{ExpenseKind, IncomeKind, Ledger, TransactionMetadata},
    utils::display_currency,
};
use num_derive::FromPrimitive;
use num::traits::FromPrimitive;
use std::borrow::Cow;
use termion::event::Key;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Paragraph, SelectableList, Tabs, Text, Widget},
};

mod delete;
mod list_nav;
mod new_account;
mod new_txn;
mod text;
mod utils;

#[derive(PartialEq, Eq)]
enum LedgerList {
    Accounts,
    Transactions,
}

/// TODO: Refactor with more data in the LedgerTabState
#[derive(PartialEq, Eq)]
enum LedgerTabState {
    Normal,
    NewAccount,
    NewTransaction(u8, NewTransactionKind),
    Delete,
}

#[derive(PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum NewTransactionKind {
    GeneralIncome = 0,
    GeneralExpense = 1,
    DonationIncome = 2,
}

pub struct LedgerTab<'a> {
    active_list: LedgerList,
    account_cursor: usize,
    accounts_cursors: Vec<usize>,
    accounts_names: Vec<String>,
    transactions_names: Vec<Vec<String>>,
    state: LedgerTabState,
    ledger: Ledger,
    ledger_path: std::path::PathBuf,

    text_input_fields: Vec<String>,
    rendered_fields: Vec<Text<'a>>,
    selected_field: usize,

    help_text: Vec<Text<'a>>,
    info_text: Vec<Text<'a>>,
}

impl<'a> LedgerTab<'a> {
    pub fn new(ledger_path: std::path::PathBuf) -> LedgerTab<'a> {
        let ledger = if !ledger_path.exists() {
            crate::ledger::new(ledger_path.clone());
            Ledger::default()
        } else {
            Ledger::load(ledger_path.clone()).unwrap_or_else(|e| panic!("Failed to load ledger!\n{}", e))
        };

        let mut new = LedgerTab {
            active_list: LedgerList::Accounts,
            account_cursor: 0,
            accounts_cursors: ledger.accounts.iter().map(|_| 0).collect(),
            accounts_names: ledger.accounts.iter().map(|x| x.name.clone()).collect(),
            transactions_names: text::generate_transaction_names(&ledger),
            state: LedgerTabState::Normal,
            ledger,
            ledger_path,

            text_input_fields: Vec::new(),
            rendered_fields: Vec::new(),
            selected_field: 0,

            help_text: Vec::new(),
            info_text: Vec::new(),
        };

        text::generate_help_text(&mut new);
        text::generate_info_text(&mut new);

        new
    }
}

impl<'a> MainTab for LedgerTab<'a> {
    fn name(&self) -> &'static str {
        "Ledger"
    }

    fn render(&self, f: &mut OrdinaryFrame, frame: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(frame);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[0]);

        let top_left_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(left_chunks[0]);

        if self.state == LedgerTabState::Delete {
            Paragraph::new(
                [
                    Text::raw("\n   Are you sure you want to delete this "),
                    if self.active_list == LedgerList::Accounts {
                        Text::styled(
                            "account",
                            Style::default().fg(Color::Red).modifier(Modifier::Bold),
                        )
                    } else {
                        Text::styled(
                            "transaction",
                            Style::default().fg(Color::Cyan).modifier(Modifier::Bold),
                        )
                    },
                    Text::raw("?\n   Press Del to confirm, or any other key to cancel."),
                ]
                .iter(),
            )
            .block(Block::default().borders(Borders::ALL).title("Delete"))
            .wrap(true)
            .render(f, top_left_chunks[0]);
        } else {
            Paragraph::new(self.help_text.iter())
                .block(Block::default().borders(Borders::ALL)) //.title("Help"))
                .wrap(true)
                .render(f, top_left_chunks[0]);

            match self.state {
                LedgerTabState::NewAccount => {
                    Paragraph::new(self.rendered_fields.iter())
                        .block(Block::default().borders(Borders::ALL).title("New account"))
                        .wrap(true)
                        .render(f, chunks[1]);
                }
                LedgerTabState::NewTransaction(_, _) => {
                    Paragraph::new(self.rendered_fields.iter())
                        .block(Block::default().borders(Borders::ALL).title("New transaction"))
                        .wrap(true)
                        .render(f, chunks[1]);
                }
                _ => {}
            }
        }

        if self.accounts_names.len() != 0 {
            SelectableList::default()
                .block(Block::default().title("Accounts").borders(Borders::ALL))
                .items(&self.accounts_names)
                .select(if self.accounts_names.len() == 0 {
                    None
                } else {
                    Some(self.account_cursor)
                })
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default().fg(if self.active_list == LedgerList::Accounts {
                        Color::Yellow
                    } else {
                        Color::Rgb(220, 140, 0)
                    }),
                )
                .render(f, top_left_chunks[1]);

            let txns = &self.transactions_names.get(self.account_cursor).expect(
                "Unreachable: transaction name out of bounds in ledger for transaction listing",
            );

            if self.state == LedgerTabState::Normal || self.state == LedgerTabState::Delete {
                if txns.len() != 0 {
                    SelectableList::default()
                        .items(txns)
                        .block(Block::default().title("Transactions").borders(Borders::ALL))
                        .select(Some(*self.accounts_cursors.get(self.account_cursor).expect(
                            "Unreachable: account cursor out of bounds in ledger for transaction listing",
                        )))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().fg(if self.active_list == LedgerList::Transactions { Color::Yellow } else { Color::Rgb(220, 140, 0) }))
                        .render(f, chunks[1]);
                } else {
                    Paragraph::new([].iter())
                        //.block(Block::default().borders(Borders::ALL))
                        .render(f, chunks[1])
                }
            }

            Paragraph::new(self.info_text.iter())
                .block(
                    Block::default().borders(Borders::ALL), //.title("Information")
                )
                .wrap(true)
                .render(f, left_chunks[1]);
        }
    }

    fn event(&mut self, event: Event<Key>) -> Trans {
        match self.state {
            LedgerTabState::Normal => list_nav::event(self, event),
            LedgerTabState::NewAccount => new_account::event(self, event),
            LedgerTabState::NewTransaction(_, _) => new_txn::event(self, event),
            LedgerTabState::Delete => delete::event(self, event),
        }
    }
}
