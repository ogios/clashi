use std::sync::LazyLock;

use crossterm::event::KeyModifiers;
use group_page::GroupPage;
use ratatui::widgets::{Tabs, Widget};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};

use crate::backend::{
    ProxyGroup, SelectableProxy, get_proxy_groups, latency_test_group, latency_test_proxy,
    select_proxy,
};

mod card;
mod group_page;
mod proxy_page;

#[derive(Debug)]
pub struct BoardWidget {
    current_tab: Tab,
    group_tab_state: ProxyTabState,
}

impl BoardWidget {
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Group,
            group_tab_state: ProxyTabState {
                groups: get_proxy_groups(),
                group_card_wdiget: GroupPage::new(4, 25),
                current_page: ProxyTabStatePage::Group,
                proxy_table: proxy_page::ProxyPage::new(),
            },
        }
    }
    pub fn draw_tab_pane(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        Tabs::new(Tab::all_names())
            .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
            .divider("|")
            .select(Tab::variants().iter().position(|n| n == &self.current_tab))
            .render(area, buf);
    }
    pub fn draw_tab(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self.current_tab {
            Tab::Group => {
                self.group_tab_state.draw(area, buf);
            }
            Tab::Provider => {}
        }
    }
    pub fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Tab => {
                self.current_tab.next();
            }
            crossterm::event::KeyCode::BackTab => {
                self.current_tab.prev();
            }
            _ => match self.current_tab {
                Tab::Group => {
                    self.group_tab_state.key_event(key);
                }
                Tab::Provider => todo!(),
            },
        }
    }
}

#[derive(Debug, IntoStaticStr, EnumIter, Eq, PartialEq, Clone, Copy)]
pub enum Tab {
    Group,
    Provider,
}
impl Tab {
    fn all_names() -> Vec<&'static str> {
        static NAMES: LazyLock<Vec<&'static str>> =
            LazyLock::new(|| Tab::iter().map(|t| t.into()).collect());
        NAMES.clone()
    }

    fn variants() -> &'static [Self] {
        static VARIANTS: LazyLock<Vec<Tab>> = LazyLock::new(|| Tab::iter().collect());
        &VARIANTS
    }

    fn next(&mut self) {
        let variants = Self::variants();
        let next_idx = (variants.iter().position(|v| v == self).unwrap() + 1) % variants.len();
        *self = variants[next_idx];
    }
    fn prev(&mut self) {
        let variants = Self::variants();
        let prev_idx = (variants.iter().position(|v| v == self).unwrap() + variants.len() - 1)
            % variants.len();
        *self = variants[prev_idx]
    }
}

#[derive(Debug)]
enum ProxyTabStatePage {
    Group,
    Proxy,
}

#[derive(Debug)]
pub struct ProxyTabState {
    groups: Vec<ProxyGroup>,
    current_page: ProxyTabStatePage,
    group_card_wdiget: GroupPage,
    proxy_table: proxy_page::ProxyPage,
}
impl ProxyTabState {
    fn get_current_group(&self) -> Option<&ProxyGroup> {
        self.groups.get(self.group_card_wdiget.get_current_item())
    }
    fn refresh(&mut self) {
        self.groups = get_proxy_groups();
    }
    fn get_current_proxy<'a>(&self, group: &'a ProxyGroup) -> Option<&'a SelectableProxy> {
        self.proxy_table
            .get_current_item()
            .map(|index| &group.proxies[index])
    }
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
                Char('r') => {
                    if let Some(g) = self.get_current_group() {
                        latency_test_group(&g.name);
                        self.refresh();
                    }
                }
                _ => {}
            },
            ProxyTabStatePage::Proxy => match key.code {
                Esc => self.current_page = ProxyTabStatePage::Group,
                Char(' ') | Enter => {
                    if let Some((g, p)) = self
                        .get_current_group()
                        .and_then(|group| self.get_current_proxy(group).map(|proxy| (group, proxy)))
                    {
                        select_proxy(&g.name, &p.name);
                        self.refresh();
                    };
                }
                Char('j') | Up => self.proxy_table.j(),
                Char('k') | Down => self.proxy_table.k(),
                Home => todo!(),
                End => todo!(),
                Char('u') if key.modifiers == KeyModifiers::CONTROL => todo!(),
                PageUp => todo!(),
                Char('d') if key.modifiers == KeyModifiers::CONTROL => todo!(),
                PageDown => todo!(),
                Char('R') => {
                    if let Some(g) = self.get_current_group() {
                        latency_test_group(&g.name);
                        self.refresh();
                    }
                }
                Char('r') => {
                    if let Some(p) = self
                        .get_current_group()
                        .and_then(|group| self.get_current_proxy(group))
                    {
                        latency_test_proxy(&p.name);
                        self.refresh();
                    };
                }
                _ => {}
            },
        }
    }
}
