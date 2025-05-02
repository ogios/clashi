use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

use crate::backend::data::ProxyGroup;

struct Card {
    current_selection: usize,

    height_of_each: u16,
    threshold_width: u16,
    row_offset: u32,
    last_select_operation: SelectOperation,
}

enum SelectOperation {
    Up,
    Down,
}

/// the number repersents how many rows to leak
enum RowPageLeak {
    /// leak a row's uppper part
    Up(Rect),
    /// leak a row's lower part
    Down(Rect),
    /// no leak
    Fit,
}

impl Card {
    fn new(height_of_each: u16, threshold_width: u16) -> Card {
        Card {
            row_offset: 0,
            current_selection: 0,
            height_of_each,
            threshold_width,
            last_select_operation: SelectOperation::Up,
        }
    }

    pub fn cal(&self, rect: Rect) -> (usize, Vec<Rect>, RowPageLeak) {}

    fn draw(&self, area: Rect, buf: &mut Buffer, data: &[&ProxyGroup]) {
        let (card_start_offset, cards_rect, row_leak) = self.cal(area);

        match row_leak {
            RowPageLeak::Up(scroll_hint) => {
                draw_scroll_hint(scroll_hint, buf, true);
            }
            RowPageLeak::Down(scroll_hint) => {
                draw_scroll_hint(scroll_hint, buf, false);
            }
            RowPageLeak::Fit => {}
        }

        for (i, card_area) in cards_rect.into_iter().enumerate() {
            let data = data[card_start_offset + i];
            draw_card_proxy_group(card_area, buf, data);
        }
    }
}

fn draw_card_proxy_group(area: Rect, buf: &mut Buffer, data: &ProxyGroup) {
    let block = Block::bordered().title_top(format!("({}){}", data.all.len(), data.now));
    Paragraph::new(data.name.clone())
        .wrap(Wrap { trim: false })
        .block(block)
        .render(area, buf);
}

fn draw_scroll_hint(area: Rect, buf: &mut Buffer, is_up: bool) {
    let mut lines: Vec<Line> = vec![
        Line::default()
            .style(Style::new().bg(ratatui::style::Color::LightBlue))
            .centered();
        area.height as usize
    ];
    if is_up {
        lines.first_mut().unwrap().push_span("⌃");
    } else {
        lines.last_mut().unwrap().push_span("⌄");
    }

    Paragraph::new(lines).render(area, buf);
}
