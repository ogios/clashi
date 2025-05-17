use std::sync::LazyLock;

use bytesize::ByteSize;
use card_page::CardPage;
use chrono::{DateTime, TimeZone, Utc};
use crossterm::event::KeyModifiers;
use humanize_duration::prelude::DurationExt;
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Tabs, Widget, Wrap},
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};
use vertical_gauge::VerticalGauge;

use crate::backend::{
    Provider, ProxyGroup, SelectableProxy, get_proxy_groups, get_proxy_providers,
    latency_test_group, latency_test_proxy, select_proxy,
};

mod card;
mod card_page;
mod provider_page;
mod proxy_page;
mod vertical_gauge;

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
pub struct BoardWidget {
    current_tab: Tab,
    group_tab_state: ProxyTabState,
    provider_tab_state: ProviderTab,
}

impl BoardWidget {
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Group,
            group_tab_state: ProxyTabState {
                groups: get_proxy_groups(),
                group_page: CardPage::new(4, 25),
                current_page: ProxyTabStatePage::Group,
                proxy_page: proxy_page::ProxyPage::new(),
            },
            provider_tab_state: ProviderTab {
                providers: get_proxy_providers(),
                current_page: ProviderTabState::Providers,
                provider_page: CardPage::new(6, 40),
                proxy_page: proxy_page::ProxyPage::new(),
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
            Tab::Group => self.group_tab_state.draw(area, buf),
            Tab::Provider => self.provider_tab_state.draw(area, buf),
        }
    }
    pub fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Tab => self.current_tab.next(),
            crossterm::event::KeyCode::BackTab => self.current_tab.prev(),
            _ => match self.current_tab {
                Tab::Group => self.group_tab_state.key_event(key),
                Tab::Provider => self.provider_tab_state.key_event(key),
            },
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
    groups: Vec<ProxyGroup>,
    current_page: ProxyTabStatePage,
    group_page: CardPage,
    proxy_page: proxy_page::ProxyPage,
}
impl ProxyTabState {
    fn get_current_group(&self) -> Option<&ProxyGroup> {
        self.groups.get(self.group_page.get_current_item())
    }
    fn refresh(&mut self) {
        self.groups = get_proxy_groups();
    }
    fn get_current_proxy<'a>(&self, group: &'a ProxyGroup) -> Option<&'a SelectableProxy> {
        self.proxy_page
            .get_current_item()
            .map(|index| &group.proxies[index])
    }
    fn draw_group_item(area: Rect, buf: &mut Buffer, data: &ProxyGroup, is_selected: bool) {
        let mut block = Block::bordered()
            .title_top({
                let ty = data.proxy_type.str().on_white().black();
                if is_selected { ty.on_green() } else { ty }
            })
            .title_top(
                Line::from(data.latency.map_or("--".to_string(), |l| format!("{l}ms")))
                    .right_aligned()
                    .bold(),
            )
            .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));

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

    fn draw(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        match self.current_page {
            ProxyTabStatePage::Group => {
                let data = &self.groups;
                self.group_page
                    .draw(area, buf, data.len(), |index, rect, buffer, state| {
                        let is_selected = index == state.get_current_item();
                        let data = &data[index];
                        Self::draw_group_item(rect, buffer, data, is_selected);
                    });
            }
            ProxyTabStatePage::Proxy => {
                let group = &self.groups[self.group_page.get_current_item()];
                let position = group
                    .proxies
                    .iter()
                    .position(|p| group.now.as_ref().is_some_and(|n| n == &p.name));
                self.proxy_page.draw(area, buf, &group.proxies, position);
            }
        }
    }
    fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode::*;

        match self.current_page {
            ProxyTabStatePage::Group => match key.code {
                Char(' ') | Enter => self.current_page = ProxyTabStatePage::Proxy,
                Char('h') | Up => self.group_page.previous_item(),
                Char('j') | Down => self.group_page.next_row(),
                Char('k') | Left => self.group_page.previous_row(),
                Char('l') | Right => self.group_page.next_item(),
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
                Char('j') | Up => self.proxy_page.j(),
                Char('k') | Down => self.proxy_page.k(),
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

#[derive(Debug)]
enum ProviderTabState {
    Providers,
    Proxy,
}
#[derive(Debug)]
pub struct ProviderTab {
    providers: Vec<Provider>,
    current_page: ProviderTabState,
    provider_page: CardPage,
    proxy_page: proxy_page::ProxyPage,
}
impl ProviderTab {
    fn get_current_provider(&self) -> Option<&Provider> {
        self.providers.get(self.provider_page.get_current_item())
    }
    fn refresh(&mut self) {
        self.providers = get_proxy_providers();
    }
    fn draw_provider_item(area: Rect, buf: &mut Buffer, data: &Provider, is_selected: bool) {
        let mut block = Block::bordered()
            .title_top({
                let ty = format!("{}({})", data.vehicle_type, data.proxies.len())
                    .on_white()
                    .black();
                if is_selected { ty.on_green() } else { ty }
            })
            .title_top(
                Line::from(
                    data.subscription_info
                        .as_ref()
                        .and_then(|i| i.expire)
                        .map_or("--".to_string(), |l| {
                            format!(
                                "expire: {}",
                                chrono::Local
                                    .timestamp_opt(l, 0)
                                    .unwrap()
                                    .format("%Y-%m-%d")
                            )
                        }),
                )
                .right_aligned()
                .bold(),
            )
            .title_bottom(format!("last update: {} ago", {
                let res = (Utc::now()
                    - DateTime::parse_from_rfc3339(&data.updated_at)
                        .unwrap()
                        .with_timezone(&Utc))
                .human(humanize_duration::Truncate::Minute)
                .to_string();
                res.split_whitespace().collect::<Vec<_>>()[0].to_string()
            }))
            .padding(ratatui::widgets::Padding::new(1, 1, 0, 0));

        if is_selected {
            block = block.green();
        }

        let inner_area = block.inner(area);
        block.render(area, buf);

        let [item_area, _, gauge_area] = Layout::horizontal([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
        .areas(inner_area);

        let ratio = data
            .subscription_info
            .as_ref()
            .filter(|info| info.total.is_some())
            .map(|info| {
                (info.upload.unwrap_or(0) + info.download.unwrap_or(0)) as f64
                    / info.total.unwrap() as f64
            })
            .unwrap_or(0.0);

        VerticalGauge::default()
            .ratio(ratio)
            .bg(Color::DarkGray)
            .render(gauge_area, buf);

        let lines = Text::from(vec![
            Line::from(data.name.clone().bold()),
            Line::from(
                data.subscription_info
                    .as_ref()
                    .filter(|info| info.total.is_some())
                    .map(|info| {
                        format!(
                            "{} / {}",
                            ByteSize::b(info.upload.unwrap_or(0) + info.download.unwrap_or(0))
                                .display()
                                .iec_short(),
                            ByteSize::b(info.total.unwrap()).display().iec_short()
                        )
                    })
                    .unwrap_or_default(),
            ),
        ]);

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(item_area, buf);
    }

    fn draw(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        use ProviderTabState::*;
        match self.current_page {
            Providers => {
                let data = &self.providers;
                self.provider_page
                    .draw(area, buf, data.len(), |index, rect, buffer, state| {
                        let is_selected = index == state.get_current_item();
                        let data = &data[index];
                        Self::draw_provider_item(rect, buffer, data, is_selected);
                    });
            }
            Proxy => {
                self.proxy_page.draw(
                    area,
                    buf,
                    &self.providers[self.provider_page.get_current_item()].proxies,
                    None,
                );
            }
        }
    }
    fn key_event(&mut self, key: crossterm::event::KeyEvent) {
        use ProviderTabState::*;
        use crossterm::event::KeyCode::*;

        match self.current_page {
            Providers => match key.code {
                Char(' ') | Enter => self.current_page = Proxy,
                Char('h') | Up => self.provider_page.previous_item(),
                Char('j') | Down => self.provider_page.next_row(),
                Char('k') | Left => self.provider_page.previous_row(),
                Char('l') | Right => self.provider_page.next_item(),
                Char('r') => {
                    if let Some(g) = self.get_current_provider() {
                        latency_test_group(&g.name);
                        self.refresh();
                    }
                }
                _ => {}
            },
            Proxy => match key.code {
                Esc => self.current_page = Providers,
                Char('j') | Up => self.proxy_page.j(),
                Char('k') | Down => self.proxy_page.k(),
                Char('R') => {
                    if let Some(g) = self.get_current_provider() {
                        latency_test_group(&g.name);
                        self.refresh();
                    }
                }
                Char('r') => {
                    todo!()
                    // if let Some(p) = self
                    //     .get_current_provider()
                    //     .and_then(|group| self.get_current_proxy(group))
                    // {
                    //     latency_test_proxy(&p.name);
                    //     self.refresh();
                    // };
                }
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
