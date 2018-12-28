use super::{LedgerList, LedgerTab, LedgerTabState};
use crate::ledger::{ExpenseKind, IncomeKind, Ledger, TransactionMetadata};
use tui::{
    style::{Color, Modifier, Style},
    widgets::Text,
};

pub fn generate_help_text(tab: &mut LedgerTab) {
    tab.help_text.clear();
    tab.help_text.push(Text::raw("\n"));

    match tab.state {
        LedgerTabState::Normal => {
            tab.help_text.push(Text::raw(
                "   Press Esc to save and quit.\n   Press Ctrl+Q to quit without saving.\n   Press Ctrl+S to save without quitting.\n   Press Tab to switch tab.\n\n   Use the arrow keys to navigate the lists.\n   Use page up and down for SANIC SPED.\n\n",
            ));
            match tab.active_list {
                LedgerList::Accounts => {
                    tab.help_text
                        .push(Text::raw("   Press + to add an account.\n"));
                    if tab.ledger.accounts.len() != 0 {
                        tab.help_text.push(Text::raw("   Press Enter to add a transaction.\n   Press - then Del to delete the selected account.\n"))
                    }
                    tab.help_text.push(Text::raw("\n"));
                }
                LedgerList::Transactions => {
                    tab.help_text.push(Text::raw("   Press + to add an account.\n   Press Enter to add a transaction.\n   Press - then Del to delete the selected transaction.\n\n"));
                }
            }
        }
        LedgerTabState::NewAccount => {
            tab.help_text
                .push(Text::raw("   Press Esc to cancel the new account."));
        }
        _ => {}
    }
}

pub fn generate_info_text(tab: &mut LedgerTab) {
    tab.info_text.clear();

    if tab.accounts_names.len() != 0 {
        tab.info_text.push(Text::raw("\n"));
        tab.info_text.push(Text::styled(
            "   Account\n",
            Style::default()
                .modifier(Modifier::Bold)
                .fg(Color::LightCyan),
        ));

        let account = tab
            .ledger
            .accounts
            .get(tab.account_cursor)
            .expect("Unreachable: account cursor out of bounds for info text");
        tab.info_text.push(Text::raw(format!(
            "   Account name: {}\n   Opening balance: ${}\n   Current balance: ${}\n\n\n",
            account.name,
            &account.opening_balance.to_string(),
            &account.current_balance().to_string(),
        )));

        if tab.active_list == LedgerList::Transactions {
            tab.info_text.push(Text::styled(
                "   Transaction\n",
                Style::default()
                    .modifier(Modifier::Bold)
                    .fg(Color::LightCyan),
            ));

            let txn_cursor = *tab
                .accounts_cursors
                .get(tab.account_cursor)
                .expect("Unreachable: account cursor out of cursors bounds for info text");
            let txn = account
                .transactions
                .get(txn_cursor)
                .expect("Unreachable: transaction out of bounds for info text");
            let txn_name = tab
                .transactions_names
                .get(tab.account_cursor)
                .expect("Unreachable: txn_name 1")
                .get(txn_cursor)
                .expect("Unreachable: txn_name 2");

            let mut fees = currency::Currency::from(0, '$');
            for x in &txn.fees {
                fees = fees + &x.amount;
            }
            
            tab.info_text.push(Text::raw(format!(
                "   {}\n   {}\n   Date: {}\n   Gross amount: ${}\n   Fees: ${}\n",
                txn_name,
                txn.description,
                txn.date.date(),
                &txn.amount.to_string(),
                &fees.to_string(),
            )));

            if let TransactionMetadata::Income {
                kind: IncomeKind::Donation(ref uuid),
                ..
            } = &txn.meta
            {
                tab.info_text.push(Text::raw(format!(
                    "   Donation ID: {}\n",
                    hex::encode(uuid)
                )));
            } else if let TransactionMetadata::Expense {
                kind: ExpenseKind::Payout(ref uuid),
                ..
            } = &txn.meta
            {
                tab.info_text.push(Text::raw(format!(
                    "   Payout ID: {}\n",
                    hex::encode(uuid)
                )));
            }
        }
    }
}

pub fn generate_transaction_names(ledger: &Ledger) -> Vec<Vec<String>> {
    ledger
        .accounts
        .iter()
        .map(|x| {
            x.transactions
                .iter()
                .map(|x| match x.meta {
                    TransactionMetadata::Income {
                        kind: IncomeKind::Donation(_),
                        ref from,
                        ..
                    } => format!("Donation from {} (${})", from, &x.amount.to_string()),
                    TransactionMetadata::Income {
                        kind: IncomeKind::General,
                        ref from,
                        ..
                    } => format!("Income from {} (${})", from, &x.amount.to_string()),
                    TransactionMetadata::Expense {
                        kind: ExpenseKind::General,
                        ref towards,
                        ref requester,
                    } => format!(
                        "Expense requested by {} paid to {} (${})",
                        requester,
                        towards,
                        &x.amount.to_string(),
                    ),
                    TransactionMetadata::Expense {
                        kind: ExpenseKind::Payout(_),
                        ref towards,
                        ..
                    } => format!(
                        "Payout to {} (${})",
                        towards,
                        &x.amount.to_string(),
                    ),
                })
                .collect()
        })
        .collect()
}

pub fn generate_input_fields_text(ledger: &mut LedgerTab, fields: &[&str], next: &'static str) {
    ledger.rendered_fields.clear();
    ledger.rendered_fields.push(Text::raw("\n"));

    use crate::utils::GetOrDefault;

    for (i, v) in fields.iter().enumerate() {
        if i == ledger.selected_field {
            ledger.rendered_fields.push(Text::styled(
                format!("   {}: {}\n", v, ledger.text_input_fields.get_or_default(i)),
                Style::default().fg(Color::Yellow),
            ));
        } else {
            ledger.rendered_fields.push(Text::raw(format!(
                "   {}: {}\n",
                v,
                ledger.text_input_fields.get_or_default(i)
            )));
        }
    }

    ledger.rendered_fields.push(Text::raw("\n   "));
    if ledger.selected_field == fields.len() {
        ledger
            .rendered_fields
            .push(Text::styled(next, Style::default().fg(Color::Yellow)));
    } else {
        ledger.rendered_fields.push(Text::raw(next));
    }
}
