use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{palette::tailwind, Stylize},
    symbols::{self},
    text::Line,
    widgets::{Block, Padding, Paragraph, Widget},
};
use strum::{Display, EnumIter, FromRepr};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum AppTab {
    #[default]
    #[strum(to_string = "Tab 1")]
    Tab1,
    #[strum(to_string = "Tab 2")]
    Tab2,
}

impl Widget for AppTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Self::Tab1 => self.render_tab0(area, buf),
            Self::Tab2 => self.render_tab1(area, buf),
        }
    }
}

impl AppTab {
    pub fn title(self) -> Line<'static> {
        format!("  {self}  ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    fn render_tab0(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Hello from Tab 0/1.")
            .block(self.block())
            .render(area, buf);
    }

    fn render_tab1(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("Hello from Tab 1/1.")
            .block(self.block())
            .render(area, buf);
    }

    fn block(self) -> Block<'static> {
        Block::bordered()
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .padding(Padding::horizontal(1))
            .border_style(self.palette().c700)
    }

    pub const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Tab1 => tailwind::GREEN,
            Self::Tab2 => tailwind::ROSE,
        }
    }

    pub fn previous(self) -> AppTab {
        let current_index = self as usize;
        let next_index = current_index.saturating_sub(1);

        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn next(self) -> AppTab {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_add(1);

        Self::from_repr(previous_index).unwrap_or(self)
    }
}
