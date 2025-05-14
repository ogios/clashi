use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
        Wrap,
    },
};

use crate::backend::ProxyGroup;

use super::card::{Card, CardState};

#[derive(Debug)]
pub struct GroupPage {
    card: Card,
    card_state: CardState,
    scroll_state: ScrollbarState,
}

impl GroupPage {
    pub fn new(height_of_each: u16, threshold_width: u16) -> GroupPage {
        GroupPage {
            card: Card::new(height_of_each, threshold_width),
            card_state: CardState::default(),
            scroll_state: ScrollbarState::default(),
        }
    }
    pub fn draw(&mut self, area: Rect, buf: &mut Buffer, data: &[ProxyGroup]) {
        let [cards_area, scrollbar_area] = Layout::horizontal([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(1),
        ])
        .areas(area);

        self.card.draw(
            &mut self.card_state,
            cards_area,
            buf,
            data.len(),
            |index, rect, buffer, state| {
                let is_selected = index == state.get_current_item();
                let data = &data[index];
                draw_card_proxy_group(rect, buffer, data, is_selected);
            },
        );

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

    pub fn h(&mut self) {
        self.card_state.previous_item();
    }
    pub fn l(&mut self) {
        self.card_state.next_item();
    }
    pub fn j(&mut self) {
        self.card_state.next_row();
    }
    pub fn k(&mut self) {
        self.card_state.previous_row();
    }
    pub fn get_current_item(&self) -> usize {
        self.card_state.get_current_item()
    }
}

fn draw_card_proxy_group(area: Rect, buf: &mut Buffer, data: &ProxyGroup, is_selected: bool) {
    let mut block = Block::bordered()
        .title_top({
            let ty = data.proxy_type.str().on_white().black();
            if is_selected { ty.on_green() } else { ty }
        })
        .title_top(
            Line::from(data.latency.map_or("--".to_string(), |l| format!("{l}ms")))
                .right_aligned()
                .bold(),
        );

    if is_selected {
        block = block.green();
    }

    if let Some(now) = data.now.as_ref() {
        block = block.title_bottom(now.to_owned().italic())
    };

    Paragraph::new(data.name.clone().bold())
        .wrap(Wrap { trim: false })
        .block(block)
        .render(area, buf);
}
