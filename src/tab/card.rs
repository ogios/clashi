use std::cell::Cell;

use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget, Wrap},
};

use crate::backend::ProxyGroup;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GroupPage {
    // card item selection, not row selection
    pub current_selection: usize,
    // start offset of the row to be displayed in the current page
    row_offset: usize,
    // controls the leak of the page
    last_select_operation: SelectOperation,

    height_of_each: u16,
    threshold_width: u16,

    // this should be set after first draw inside `cal`
    cards_in_a_row: Cell<Option<usize>>,
    rows: Cell<Option<u16>>,
    max_item_num: Cell<Option<usize>>,
}

impl GroupPage {
    pub fn new(height_of_each: u16, threshold_width: u16) -> GroupPage {
        GroupPage {
            row_offset: 0,
            current_selection: 0,
            height_of_each,
            threshold_width,
            last_select_operation: SelectOperation::Up,
            cards_in_a_row: Cell::new(None),
            rows: Cell::new(None),
            max_item_num: Cell::new(None),
        }
    }

    fn get_item_num_range(&self) -> (usize, usize) {
        let cards_in_a_row = self.cards_in_a_row.get().unwrap();
        let rows = self.rows.get().unwrap();
        let start_offset = self.row_offset * cards_in_a_row;
        let end_offset = start_offset + (rows as usize) * cards_in_a_row;
        (start_offset, end_offset)
    }

    fn check_if_within_page(&mut self) {
        // check if within the page
        let range = self.get_item_num_range();
        if self.current_selection >= range.1 {
            // move to the right
            self.row_offset += 1;
            self.last_select_operation = SelectOperation::Down;
        } else if self.current_selection < range.0 {
            // move to the left
            self.row_offset = self.row_offset.saturating_sub(1); // NOTE: assume this is the same as max(0, num)
            self.last_select_operation = SelectOperation::Up;
        }
    }

    /// left selection
    pub fn h(&mut self) {
        self.current_selection = self.current_selection.saturating_sub(1);
        self.check_if_within_page();
    }

    pub fn l(&mut self) {
        let max_num = self.max_item_num.get().unwrap() - 1;
        let select = self.current_selection + 1;
        self.current_selection = max_num.min(select);
        self.check_if_within_page();
    }

    pub fn j(&mut self) {
        let max_num = self.max_item_num.get().unwrap() - 1;
        let select = self.current_selection + self.cards_in_a_row.get().unwrap();
        self.current_selection = max_num.min(select);
        self.check_if_within_page();
    }

    // HAVE NOT CHECKED
    pub fn k(&mut self) {
        self.current_selection = 0
            .max(self.current_selection as isize - self.cards_in_a_row.get().unwrap() as isize)
            as usize;
        self.check_if_within_page();
    }

    fn cal(&self, rect: Rect) -> (usize, Vec<Rect>, RowPageLeak) {
        // how many rows can be displayed
        let rows = rect.height / self.height_of_each;
        // how many leaks lines
        let leaks = rect.height % self.height_of_each;

        let (row_page_leak, cards_area) = if leaks == 0 {
            // no leak
            (RowPageLeak::Fit, rect)
        } else {
            let mut constrain = [
                ratatui::layout::Constraint::Percentage(100),
                ratatui::layout::Constraint::Length(leaks),
            ];

            if let SelectOperation::Up = self.last_select_operation {
                let [cards_area, leak_rect] = Layout::vertical(constrain).areas(rect);
                (RowPageLeak::Down(leak_rect), cards_area)
            } else {
                constrain.reverse();
                let [cards_area, leak_rect] = Layout::vertical(constrain).areas(rect);
                (RowPageLeak::Up(leak_rect), cards_area)
            }
        };

        // how many card in a row
        let cards_in_a_row = (cards_area.width / self.threshold_width) as usize;
        self.cards_in_a_row.set(Some(cards_in_a_row));

        // item start index
        let start_offset = self.row_offset * cards_in_a_row;

        // PERF: THIS MAY IMPORVE PERFORMANCE IDK
        // let card_num = (rows as usize * cards_in_a_row).min(items_len - start_offset);
        // // actual rows for item render
        // let actual_rows = (card_num + (cards_in_a_row - 1)) / cards_in_a_row as usize; // do ceil and avoid converting to float

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

        self.rows.set(Some(rows));

        (start_offset, render_rows, row_page_leak)
    }

    pub fn draw(&self, area: Rect, buf: &mut Buffer, data: &[ProxyGroup]) {
        self.max_item_num.set(Some(data.len()));
        let (card_start_offset, cards_rect, row_leak) = self.cal(area);

        for (i, card_area) in cards_rect.into_iter().enumerate() {
            let index = card_start_offset + i;
            if index >= data.len() {
                break;
            }
            let data = &data[index];
            draw_card_proxy_group(card_area, buf, data, index == self.current_selection);
        }

        match row_leak {
            RowPageLeak::Up(scroll_hint) => {
                draw_scroll_hint(scroll_hint, buf, true);
            }
            RowPageLeak::Down(scroll_hint) => {
                draw_scroll_hint(scroll_hint, buf, false);
            }
            RowPageLeak::Fit => {}
        }
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

fn draw_scroll_hint(area: Rect, buf: &mut Buffer, is_up: bool) {
    let mut lines: Vec<Line> = vec![Line::default(); area.height as usize];
    if is_up {
        lines.first_mut().unwrap().push_span("⌃");
    } else {
        lines.last_mut().unwrap().push_span("⌄");
    }

    Paragraph::new(lines)
        .style(Style::new().on_light_blue())
        .centered()
        .render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_cal() {
        // Create a Card instance with mock parameters
        let card = GroupPage::new(4, 10);

        // Mock a Rect with specific dimensions
        let rect = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 12,
        };

        // Call the cal function
        let (start_offset, render_rows, row_page_leak) = card.cal(rect);

        // Assert the start_offset
        assert_eq!(start_offset, 0);

        // Assert the number of render_rows
        assert_eq!(render_rows.len(), 9); // 3 rows * 3 cards in a row

        // Assert the row_page_leak
        match row_page_leak {
            RowPageLeak::Fit => {} // Expected since height is perfectly divisible
            _ => panic!("Expected RowPageLeak::Fit"),
        }

        // Test with a Rect that causes a leak
        let rect_with_leak = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 11, // Causes a leak (11 % 3 != 0)
        };

        let (_, _, row_page_leak_with_leak) = card.cal(rect_with_leak);

        // Assert the row_page_leak for the leaking case
        match row_page_leak_with_leak {
            RowPageLeak::Up(_) | RowPageLeak::Down(_) => {} // Expected since height is not perfectly divisible
            _ => panic!("Expected RowPageLeak::Up or RowPageLeak::Down"),
        }
    }
}
