use std::{io, process};

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Layout, Rect},
    text::Text,
    widgets::{Paragraph, Widget},
};
use tab::BoardWidget;

mod backend;
mod tab;

#[derive(Debug)]
pub struct App {
    board_state: BoardWidget,
    counter: u8,
    exit: bool,
}
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [tab_pane, tab, bar] = Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(1),
        ])
        .areas(area);

        self.board_state.draw_tab_pane(tab_pane, buf);
        self.board_state.draw_tab(tab, buf);
        self.draw_bar(bar, buf);

        // let title = Line::from(" Counter App Tutorial ".bold());
        // let instructions = Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q> ".blue().bold(),
        // ]);
        // let block = Block::bordered()
        //     .title(title.centered())
        //     .title_bottom(instructions.centered())
        //     .border_set(border::THICK);
        //
        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);
        //
        // Paragraph::new(counter_text)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);
    }
}
impl App {
    fn new() -> Self {
        Self {
            board_state: BoardWidget::new(),
            counter: 0,
            exit: false,
        }
    }
    fn draw_bar(&self, area: Rect, buf: &mut Buffer) {
        let [keys, time] = Layout::horizontal([
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(5),
        ])
        .areas(area);

        // keyboard shortcuts
        Paragraph::new(Text::styled(
            "No keys for now",
            ratatui::style::Style::default().fg(ratatui::style::Color::Green),
        ))
        .render(keys, buf);

        // current time
        Paragraph::new(Text::styled(
            chrono::Local::now().format("%H:%M").to_string(),
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(ratatui::style::Color::Green),
        ))
        .render(time, buf);
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&self, key_event: event::KeyEvent) {
        todo!()
    }
}

fn main() -> io::Result<()> {
    // request for localhost:9090/proxies, and convert to backend::data::Root
    // let response: backend::data::Root = reqwest::blocking::get("http://localhost:9090/proxies")
    //     .unwrap()
    //     .json()
    //     .unwrap();
    // println!("{:?}", response);
    // process::exit(0);

    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);

    ratatui::restore();
    app_result
}
