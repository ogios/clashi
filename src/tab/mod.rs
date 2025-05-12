use card::Card;
use ratatui::widgets::{Tabs, Widget};

use crate::backend::{data::ProxyGroup, get_proxy_groups};

mod card;

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
                groups: get_proxy_groups(),
                group_card_wdiget: Card::new(4, 25),
                current_page: ProxyTabStatePage::Group,
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
    pub fn draw_tab(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.tabs[self.current_tab].draw(area, buf);
    }
    pub fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Tab => {
                self.current_tab = (self.current_tab + 1) % self.tabs.len();
            }
            crossterm::event::KeyCode::BackTab => {
                self.current_tab = (self.current_tab + self.tabs.len() - 1) % self.tabs.len();
            }
            _ => {
                self.tabs[self.current_tab].key_event(key);
            }
        }
    }
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
    fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        match self {
            Tab::Proxy(state) => state.key_event(key),
        }
    }
}

#[derive(Debug)]
enum ProxyTabStatePage {
    Group,
    Proxy,
}

#[derive(Debug)]
pub struct ProxyTabState {
    name: String,
    groups: Vec<ProxyGroup>,
    group_card_wdiget: Card,
    current_page: ProxyTabStatePage,
}
impl ProxyTabState {
    fn draw(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.group_card_wdiget.draw(area, buf, &self.groups);
    }
    fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        match self.current_page {
            ProxyTabStatePage::Group => {
                if key.code == crossterm::event::KeyCode::Enter {
                    self.current_page = ProxyTabStatePage::Proxy;
                } else {
                    self.group_card_wdiget.key_event(key);
                }
            }
            ProxyTabStatePage::Proxy => {
                if key.code == crossterm::event::KeyCode::Esc {
                    self.current_page = ProxyTabStatePage::Group;
                } else {
                    todo!()
                }
            }
        }
    }
}
