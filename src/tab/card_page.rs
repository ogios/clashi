use std::ops::{Deref, DerefMut};

use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};

use super::card::{Card, CardState};

#[derive(Debug)]
pub struct CardPage {
    card: Card,
    card_state: CardState,
    scroll_state: ScrollbarState,
}

impl CardPage {
    pub fn new(height_of_each: u16, threshold_width: u16) -> CardPage {
        CardPage {
            card: Card::new(height_of_each, threshold_width),
            card_state: CardState::default(),
            scroll_state: ScrollbarState::default(),
        }
    }
    pub fn draw(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        max_len: usize,
        cb: impl FnMut(usize, Rect, &mut Buffer, &mut CardState),
    ) {
        let [cards_area, scrollbar_area] = Layout::horizontal([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(1),
        ])
        .areas(area);

        self.card
            .draw(&mut self.card_state, cards_area, buf, max_len, cb);

        self.scroll_state = self
            .scroll_state
            .content_length(self.card_state.get_total_rows_count())
            .position(self.card_state.get_current_row());

        StatefulWidget::render(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .thumb_style(Style::default().fg(ratatui::style::Color::Green))
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            scrollbar_area,
            buf,
            &mut self.scroll_state,
        );
    }
}

impl Deref for CardPage {
    type Target = CardState;

    fn deref(&self) -> &Self::Target {
        &self.card_state
    }
}
impl DerefMut for CardPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.card_state
    }
}
