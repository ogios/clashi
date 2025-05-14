use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

use crate::backend::ProxyGroup;

#[derive(Debug)]
pub struct Card {
    height_of_each: u16,
    threshold_width: u16,
}

impl Card {
    pub fn new(height_of_each: u16, threshold_width: u16) -> Self {
        Self {
            height_of_each,
            threshold_width,
        }
    }

    fn calculate(&self, rect: Rect, state: &mut CardState) -> (usize, Vec<Rect>, Option<Rect>) {
        // how many rows can be displayed
        let rows = rect.height / self.height_of_each;
        // how many leaked lines
        let leaks = rect.height % self.height_of_each;

        let (row_page_leak, cards_area) = if leaks == 0 {
            // no leak
            (None, rect)
        } else {
            // always leak down
            let [cards_area, leak_rect] = Layout::vertical([
                ratatui::layout::Constraint::Percentage(100),
                ratatui::layout::Constraint::Length(leaks),
            ])
            .areas(rect);
            (Some(leak_rect), cards_area)
        };

        let cards_in_a_row = (cards_area.width / self.threshold_width) as usize;
        state.set_cards_in_a_row(cards_in_a_row);

        // item start index
        let start_offset = state.row_offset * cards_in_a_row;

        let render_rows: Vec<Rect> = Layout::vertical(
            std::iter::repeat(ratatui::layout::Constraint::Length(self.height_of_each))
                .take(rows as usize),
        )
        .split(cards_area)
        .iter()
        .copied()
        .flat_map(|row_rect| {
            Layout::horizontal(
                std::iter::repeat(ratatui::layout::Constraint::Ratio(1, cards_in_a_row as u32))
                    .take(cards_in_a_row),
            )
            .split(row_rect)
            .to_vec()
        })
        .collect();

        state.set_rows(rows);

        (start_offset, render_rows, row_page_leak)
    }

    pub fn draw(
        &self,
        state: &mut CardState,
        area: Rect,
        buf: &mut Buffer,
        item_num: usize,
        mut draw_cell: impl FnMut(usize, Rect, &mut Buffer, &mut CardState),
    ) {
        state.set_max_item_num(item_num);

        let (card_start_offset, cards_rect, row_leak) = self.calculate(area, state);

        // if all cards can be displayed, no need to draw scroll hint
        if card_start_offset + cards_rect.len() < item_num {
            if let Some(rect) = row_leak {
                draw_scroll_hint(rect, buf);
            }
        }

        for (i, card_area) in cards_rect.into_iter().enumerate() {
            let index = card_start_offset + i;
            if index >= item_num {
                break;
            }
            draw_cell(index, card_area, buf, state);
        }
    }
}

#[derive(Debug, Default)]
pub struct CardState {
    current_selection: usize,
    row_offset: usize,

    cards_in_a_row: Option<usize>,
    rows: Option<u16>,
    max_item_num: Option<usize>,
}

impl CardState {
    fn set_cards_in_a_row(&mut self, cards_in_a_row: usize) {
        self.cards_in_a_row = Some(cards_in_a_row);
    }
    fn set_rows(&mut self, rows: u16) {
        self.rows = Some(rows);
    }
    fn set_max_item_num(&mut self, max_item_num: usize) {
        self.max_item_num = Some(max_item_num);
    }

    fn get_item_num_range(&self) -> (usize, usize) {
        let cards_in_a_row = self.cards_in_a_row.unwrap();
        let rows = self.rows.unwrap();
        let start_offset = self.row_offset * cards_in_a_row;
        let end_offset = start_offset + (rows as usize) * cards_in_a_row;
        (start_offset, end_offset)
    }

    fn check_if_within_page(&mut self) {
        let range = self.get_item_num_range();
        if self.current_selection >= range.1 {
            self.row_offset += 1;
        } else if self.current_selection < range.0 {
            self.row_offset = self.row_offset.saturating_sub(1);
        }
    }

    pub fn previous_item(&mut self) {
        self.current_selection = self.current_selection.saturating_sub(1);
        self.check_if_within_page();
    }

    pub fn next_item(&mut self) {
        let max_num = self.max_item_num.unwrap() - 1;
        let select = self.current_selection + 1;
        self.current_selection = max_num.min(select);
        self.check_if_within_page();
    }

    pub fn next_row(&mut self) {
        let max_num = self.max_item_num.unwrap() - 1;
        let select = self.current_selection + self.cards_in_a_row.unwrap();
        self.current_selection = max_num.min(select);
        self.check_if_within_page();
    }

    pub fn previous_row(&mut self) {
        self.current_selection =
            0.max(self.current_selection as isize - self.cards_in_a_row.unwrap() as isize) as usize;
        self.check_if_within_page();
    }

    pub fn get_current_item(&self) -> usize {
        self.current_selection
    }
    pub fn get_current_row(&self) -> usize {
        let row_item_num = self.cards_in_a_row.unwrap();
        (self.current_selection + row_item_num - 1) / row_item_num // avoid float operation
    }
}

fn draw_scroll_hint(area: Rect, buf: &mut Buffer) {
    let mut lines: Vec<Line> = vec![Line::default(); area.height as usize];
    lines.last_mut().unwrap().push_span("⌄");
    Paragraph::new(lines)
        .style(Style::new().on_light_blue())
        .centered()
        .render(area, buf);
}
