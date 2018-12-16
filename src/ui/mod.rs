use crate::ledger::Ledger;
use std::error::Error;
use std::io;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Tabs, Widget},
};

pub const DATE_FORMAT: &'static str = "%Y/%m/%d %H:%M";

mod budget;
mod ledger;
mod tui_utils;

use self::tui_utils::{Event, Events};

pub enum Trans {
    None,
    Quit,
}

pub type OrdinaryFrame<'a> = Frame<
    'a,
    TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<io::Stdout>>>>,
>;

pub trait MainTab {
    fn name(&self) -> &'static str;
    fn render(&self, f: &mut OrdinaryFrame, frame: Rect);
    fn event(&mut self, event: Event<Key>) -> Trans;
}

struct App {
    current_tab: usize,
    tabs: Vec<Box<dyn MainTab>>,
    tab_names: Vec<&'static str>,
}

impl App {
    fn new(tabs: Vec<Box<dyn MainTab>>) -> Self {
        App {
            current_tab: 0,
            tab_names: tabs.iter().map(|x| x.name()).collect(),
            tabs,
        }
    }
}

/// This is magic stdout stuff. I understand half of what I'm doing, but it works, so whatever I guess...
pub fn start_handle_panic(ledger: Option<std::path::PathBuf>) -> Result<(), Box<dyn Error>> {
    let def_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |x| {
        print!("{}[2J", 27 as char); // Clear the screen
        println!("An error occured.");
        def_hook(x);
        use termion::input::TermRead;
        std::io::stdin().events().next();
    }));
    let res = start(ledger);
    println!("Terminated.");
    std::thread::sleep(std::time::Duration::from_millis(1000));
    res
}

pub fn start(ledger: Option<std::path::PathBuf>) -> Result<(), Box<dyn Error>> {
    // <>
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    let mut tabs: Vec<Box<dyn MainTab>> = Vec::new();

    if let Some(ledger) = ledger {
        tabs.push(Box::new(self::ledger::LedgerTab::new(ledger)));
    }

    let mut app = App::new(tabs);

    let mut size = terminal.size()?;
    loop {
        let new_size_candidate = terminal.size()?;
        if size != new_size_candidate {
            terminal.resize(new_size_candidate)?;
            size = new_size_candidate;
        }
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);
            Tabs::default()
                .block(Block::default().borders(Borders::ALL))
                .titles(&app.tab_names)
                .style(Style::default().fg(Color::Green))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.current_tab)
                .render(&mut f, chunks[0]);
            app.tabs
                .get(app.current_tab)
                .expect("Unreachable: Current tab is out of bound for rendering")
                .render(&mut f, chunks[1]);
        })?;

        match events.next()? {
            Event::Input(Key::Char('\t')) => {
                app.current_tab = if app.current_tab == app.tabs.len() - 1 {
                    0
                } else {
                    app.current_tab + 1
                };
            }
            x => {
                match app
                    .tabs
                    .get_mut(app.current_tab)
                    .expect("Unreachable: Current tab is out of bound for events")
                    .event(x)
                {
                    Trans::None => {}
                    Trans::Quit => {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
