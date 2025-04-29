use ratatui::widgets::{Tabs, Widget};

#[derive(Debug)]
pub struct BoardWidget {
    current_tab: usize,
    tabs: Box<[Tab]>,
}

impl BoardWidget {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            tabs: Box::new([Tab::Proxy(ProxyTabState {
                name: "Proxy".to_string(),
            })]),
        }
    }
    pub fn draw_tab_pane(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let tab_titles: Vec<&str> = self.tabs.iter().map(|tab| tab.name()).collect();
        let tabs = Tabs::new(tab_titles)
            .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
            .divider("|")
            .select(self.current_tab);

        tabs.render(area, buf);
    }
    pub fn draw_tab(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {}
}

#[derive(Debug)]
pub enum Tab {
    Proxy(ProxyTabState),
}
impl Tab {
    fn name(&self) -> &str {
        match self {
            Tab::Proxy(state) => &state.name,
        }
    }
    fn draw(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self {
            Tab::Proxy(state) => state.draw(area, buf),
        }
    }
}

#[derive(Debug)]
pub struct ProxyTabState {
    name: String,
}
impl ProxyTabState {
    fn draw(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {}
}
