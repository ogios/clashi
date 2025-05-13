use card::Card;
use crossterm::event::KeyModifiers;
use ratatui::widgets::{Tabs, Widget};

use crate::backend::{ProxyGroup, get_proxy_groups};

mod card;
mod proxy_table;

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
                proxy_table: proxy_table::ProxyPage::new(),
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
    pub fn draw_tab(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
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
    fn draw(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
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
    current_page: ProxyTabStatePage,
    group_card_wdiget: Card,
    proxy_table: proxy_table::ProxyPage,
}
impl ProxyTabState {
    fn draw(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self.current_page {
            ProxyTabStatePage::Group => {
                self.group_card_wdiget.draw(area, buf, &self.groups);
            }
            ProxyTabStatePage::Proxy => {
                self.proxy_table.draw(
                    area,
                    buf,
                    &self.groups[self.group_card_wdiget.current_selection],
                );
            }
        }
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
                use crossterm::event::KeyCode::*;
                match key.code {
                    Enter | Char(' ') => todo!(),
                    Up | Char('j') => self.proxy_table.j(),
                    Down | Char('k') => self.proxy_table.k(),
                    Home => todo!(),
                    End => todo!(),
                    Char('u') if key.modifiers == KeyModifiers::CONTROL => todo!(),
                    PageUp => todo!(),
                    Char('d') if key.modifiers == KeyModifiers::CONTROL => todo!(),
                    PageDown => todo!(),
                    Esc => self.current_page = ProxyTabStatePage::Group,
                    _ => {}
                }
            }
        }
    }
}
