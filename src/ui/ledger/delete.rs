use super::{
    super::tui_utils::Event,
    text::{generate_help_text, generate_info_text},
    LedgerList, LedgerTab, LedgerTabState, Trans,
};
use termion::event::Key;

pub fn event(tab: &mut LedgerTab, event: Event<Key>) -> Trans {
    if event == Event::Input(Key::Delete) {
        if tab.active_list == LedgerList::Accounts {
            tab.ledger.remove_account_at(tab.account_cursor);
            tab.accounts_cursors.remove(tab.account_cursor);
            tab.accounts_names.remove(tab.account_cursor);
            tab.transactions_names.remove(tab.account_cursor);
            if tab.account_cursor != 0 {
                tab.account_cursor -= 1;
            }
        } else {
            let account = tab
                .ledger
                .accounts
                .get_mut(tab.account_cursor)
                .expect("Unreachable: txn del acc cursor bounds");
            let cursor = tab
                .accounts_cursors
                .get_mut(tab.account_cursor)
                .expect("Unreachable: txn cursor bounds");
            let names = tab
                .transactions_names
                .get_mut(tab.account_cursor)
                .expect("Unreachable: txn names cursor bounds");
            account.transactions.remove(*cursor);
            names.remove(*cursor);
            if *cursor != 0 {
                *cursor -= 1;
            }
            if names.len() == 0 {
                tab.active_list = LedgerList::Accounts;
            }
        }
        tab.state = LedgerTabState::Normal;
        generate_help_text(tab);
        generate_info_text(tab);
    } else if event != Event::Tick {
        tab.state = LedgerTabState::Normal;
    }
    Trans::None
}
