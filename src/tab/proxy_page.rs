use ratatui::{
    layout::Layout,
    style::{Modifier, Style, Stylize},
    widgets::{
        Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState,
    },
};

use crate::backend::ProxyGroup;

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
        group: &ProxyGroup,
    ) {
        let rows: Vec<Row> = group
            .proxies
            .iter()
            .map(|p| {
                let row = Row::new(vec![
                    p.name.clone(),
                    p.proxy_type.str().to_owned(),
                    p.latency.map_or("--".to_string(), |l| format!("{l}ms")),
                    p.udp.to_string(),
                ]);
                if group.now.as_ref().is_some_and(|now| now == &p.name) {
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
        group: &ProxyGroup,
    ) {
        self.scroll_state = self.scroll_state.content_length(group.proxies.len());
        StatefulWidget::render(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area,
            buf,
            &mut self.scroll_state,
        );
    }

    pub fn draw(
        &mut self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        group: &ProxyGroup,
    ) {
        if group.proxies.is_empty() {
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
        self.draw_table(table_area, buf, group);
        self.draw_scrollbar(scrollbar_area, buf, group);
    }
}
