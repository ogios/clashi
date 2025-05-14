use crossterm::event::KeyModifiers;
use group_page::GroupPage;
use ratatui::widgets::{Tabs, Widget};

use crate::backend::{ProxyGroup, get_proxy_groups};

mod card;
mod group_page;
mod proxy_page;

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
                group_card_wdiget: GroupPage::new(4, 25),
                current_page: ProxyTabStatePage::Group,
                proxy_table: proxy_page::ProxyPage::new(),
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
    group_card_wdiget: GroupPage,
    proxy_table: proxy_page::ProxyPage,
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
                    &self.groups[self.group_card_wdiget.get_current_item()],
                );
            }
        }
    }
    fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode::*;

        match self.current_page {
            ProxyTabStatePage::Group => match key.code {
                Char(' ') | Enter => self.current_page = ProxyTabStatePage::Proxy,
                Char('h') | Up => self.group_card_wdiget.h(),
                Char('j') | Down => self.group_card_wdiget.j(),
                Char('k') | Left => self.group_card_wdiget.k(),
                Char('l') | Right => self.group_card_wdiget.l(),
                _ => {}
            },
            ProxyTabStatePage::Proxy => match key.code {
                Esc => self.current_page = ProxyTabStatePage::Group,
                Char(' ') | Enter => todo!(),
                Char('j') | Up => self.proxy_table.j(),
                Char('k') | Down => self.proxy_table.k(),
                Home => todo!(),
                End => todo!(),
                Char('u') if key.modifiers == KeyModifiers::CONTROL => todo!(),
                PageUp => todo!(),
                Char('d') if key.modifiers == KeyModifiers::CONTROL => todo!(),
                PageDown => todo!(),
                _ => {}
            },
        }
    }
}
