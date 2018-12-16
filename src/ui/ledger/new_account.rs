use super::{
    super::tui_utils::Event,
    text::{generate_help_text, generate_info_text, generate_input_fields_text},
    utils::{decrease_modular, increase_modular},
    LedgerTab, LedgerTabState, Trans,
};
use termion::event::Key;

pub const FIELDS: &'static [&'static str] = &["Name", "Opening UTC date", "Opening balance"];

pub fn event(tab: &mut LedgerTab, event: Event<Key>) -> Trans {
    match event {
        Event::Input(Key::Esc) => {
            tab.text_input_fields.clear();
            tab.selected_field = 0;
            tab.state = LedgerTabState::Normal;
        }
        Event::Input(Key::Up) => {
            decrease_modular(&mut tab.selected_field, 1, FIELDS.len() + 1);
            generate_input_fields_text(tab, FIELDS, "Confirm");
        }
        Event::Input(Key::Down) => {
            increase_modular(&mut tab.selected_field, 1, FIELDS.len() + 1);
            generate_input_fields_text(tab, FIELDS, "Confirm");
        }
        Event::Input(Key::Char('\n')) => {
            if tab.selected_field == FIELDS.len() {
                use chrono::offset::TimeZone;
                if let Ok(date) = chrono::Utc.datetime_from_str(
                    &tab.text_input_fields
                        .get(1)
                        .expect("Unreachable: new_account field1"),
                    crate::DATE_FORMAT,
                ) {
                    if let Ok(amount) = tab
                        .text_input_fields
                        .get(2)
                        .expect("Unreachable: new_account field2")
                        .parse::<f64>()
                    {
                        if let Some(amount) = num::BigRational::from_float(amount) {
                            let name = tab
                                .text_input_fields
                                .get(0)
                                .expect("Unreachable: new_account field0");
                            if name.trim() != ""
                                && !tab.ledger.accounts.iter().any(|x| x.name == *name)
                            {
                                tab.ledger.new_account(name, amount, date);
                                tab.accounts_cursors.push(0);
                                tab.transactions_names.push(Vec::new());
                                tab.accounts_names.push(name.clone());
                                tab.text_input_fields.clear();
                                tab.selected_field = 0;
                                tab.state = LedgerTabState::Normal;
                                generate_info_text(tab);
                                generate_help_text(tab);
                            }
                        }
                    }
                }
            }
        }
        Event::Input(Key::Char(x)) => {
            if let Some(field) = tab.text_input_fields.get_mut(tab.selected_field) {
                field.push(x);
                generate_input_fields_text(tab, FIELDS, "Confirm");
            }
        }
        Event::Input(Key::Backspace) => {
            if let Some(field) = tab.text_input_fields.get_mut(tab.selected_field) {
                field.pop();
                generate_input_fields_text(tab, FIELDS, "Confirm");
            }
        }
        _ => {}
    }
    Trans::None
}
