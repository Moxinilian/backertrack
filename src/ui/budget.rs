use super::{tui_utils::Event, MainTab, OrdinaryFrame, Trans};
use termion::event::Key;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Paragraph, SelectableList, Tabs, Text, Widget},
};

#[derive(Default)]
pub struct BudgetTab;

impl MainTab for BudgetTab {
    fn name(&self) -> &'static str {
        "Budget"
    }

    fn render(&self, f: &mut OrdinaryFrame, frame: Rect) {}

    fn event(&mut self, event: Event<Key>) -> Trans { Trans::None }
}
