use ratatui::{
    layout::Layout,
    style::{Modifier, Style, Stylize},
    widgets::{Row, Scrollbar, ScrollbarState, StatefulWidget, Table, TableState},
};

use crate::backend::SelectableProxy;

#[derive(Debug)]
pub struct ProxyPage {
    state: TableState,
    scroll_state: ScrollbarState,
}

impl ProxyPage {
    pub fn new() -> Self {
        Self {
            state: TableState::new().with_selected(0),
            scroll_state: ScrollbarState::default(),
        }
    }
    pub fn get_current_item(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn j(&mut self) {
        self.state.select_next();
        self.scroll_state = self.scroll_state.position(self.state.selected().unwrap());
    }

    pub fn k(&mut self) {
        self.state.select_previous();
        self.scroll_state = self.scroll_state.position(self.state.selected().unwrap());
    }

    fn draw_table(
        &mut self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        proxies: &[SelectableProxy],
        selected: Option<usize>,
    ) {
        let rows: Vec<Row> = proxies
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let row = Row::new(vec![
                    p.name.clone(),
                    p.proxy_type.str().to_owned(),
                    p.latency.map_or("--".to_string(), |l| format!("{l}ms")),
                    p.udp.to_string(),
                ]);
                if selected.is_some_and(|s| s == i) {
                    row.on_green().black()
                } else {
                    row
                }
            })
            .collect();
        let widths = vec![
            ratatui::layout::Constraint::Fill(2),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Fill(1),
            ratatui::layout::Constraint::Length(5),
        ];

        StatefulWidget::render(
            Table::new(rows, widths)
                .header(
                    Row::new(vec!["Name", "Type", "Latency", "UDP"])
                        .bold()
                        .bottom_margin(1),
                )
                .row_highlight_style(Style::new().reversed())
                .highlight_symbol(">>"),
            area,
            buf,
            &mut self.state,
        );
    }

    fn draw_scrollbar(
        &mut self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        proxies: &[SelectableProxy],
    ) {
        self.scroll_state = self.scroll_state.content_length(proxies.len());
        StatefulWidget::render(
            Scrollbar::default()
                .thumb_style(Style::default().fg(ratatui::style::Color::Green))
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            buf,
            &mut self.scroll_state,
        );
    }

    pub fn draw(
        &mut self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        proxies: &[SelectableProxy],
        selected: Option<usize>,
    ) {
        if proxies.is_empty() {
            buf.set_string(
                area.left(),
                area.top(),
                "No proxies available",
                Style::default().add_modifier(Modifier::BOLD),
            );
            return;
        }

        let [table_area, scrollbar_area] = Layout::horizontal([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(1),
        ])
        .areas(area);
        self.draw_table(table_area, buf, proxies, selected);
        self.draw_scrollbar(scrollbar_area, buf, proxies);
    }
}
