use std::{io, sync::Arc};

use crate::tabs::AppTab;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use libalembic::msg::client_server::ClientServerMessage;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Tabs, Widget},
};
use strum::IntoEnumIterator;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, error::TryRecvError},
};

pub struct App {
    title: String,
    exit: bool,
    selected_tab: AppTab,
    client_server_rx: Arc<Mutex<Receiver<ClientServerMessage>>>,
}

impl App {
    pub fn new(client_server_rx: Arc<Mutex<Receiver<ClientServerMessage>>>) -> Self {
        Self {
            title: "Alembic".to_string(),
            exit: false,
            selected_tab: AppTab::Tab1,
            client_server_rx,
        }
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            loop {
                match self.client_server_rx.try_lock().unwrap().try_recv() {
                    Ok(_) => todo!(),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        eprintln!("Channel disconnected");
                        break;
                    }
                }
            }
            terminal.draw(|frame| {
                self.draw(frame);
            })?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        self.title.clone().bold().render(area, buf);
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = AppTab::iter().map(AppTab::title);
        let highlight_style = (Color::default(), self.selected_tab.palette().c700);
        let selected_tab_index = self.selected_tab as usize;
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(selected_tab_index)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }

    fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0)]);
        let [header_area, inner_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(20)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        self.render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);
        self.selected_tab.render(inner_area, buf);

        // TODO
        // render_footer(footer_area, buf);
    }
}
