use std::fs;
use std::io;
use std::path::PathBuf;

use crate::player::Player;

use crate::config::Config;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        Block, List, ListItem, ListState, StatefulWidget, Widget,
    },
    DefaultTerminal, Frame,
};

#[derive(Debug)]
pub struct App {
    exit: bool,
    list_state: ListState,
    items: Vec<PathBuf>,
    player: Player,
}

impl App {
    pub fn new(config: Config) -> io::Result<Self> {

        // tutki music dir ja lisää ne widgettin
        let items = if let Ok(entries) = fs::read_dir(&config.music_directory) {
            entries
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<PathBuf>>()
        } else {
            Vec::new()
        }; 


        // tämänhetkinen musiikki
        let selected = if items.is_empty() {
            None
        } else {
            Some(0)
        };

        let player = Player::new()?;

        // init
        Ok(Self {
            exit: false,
            list_state: ListState::default().with_selected(selected),
            items,
            player,
        }) 
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut *self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        match key.code {
            KeyCode::Enter => {
                if let Some(index) = self.list_state.selected() {
                    if let Some(selected_item) = self.items.get(index) {
                        self.player.play(selected_item)?;
                    }
                } else {
                    println!("Nothing is currently selected");
                }
            }

            KeyCode::Up => {
                self.list_state.select_previous();
            }

            KeyCode::Down => {
                self.list_state.select_next();
            }

            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.exit = true;
            }

            KeyCode::Char(' ')  => {
                let _ = self.player.pause();
            }

            _ => {}
        }
    }

    Ok(())
} 
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);

        let items = self
        .items
        .iter()
        .map(|path| {
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            ListItem::new(filename.to_string())
        })
        .collect::<Vec<_>>();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(Line::from(" Songs ".bold()).centered())
                    .border_set(border::PLAIN),
            )
            .highlight_style(Style::default().reversed())
            .highlight_symbol(">> ");

        StatefulWidget::render(
            list,
            layout[0],
            buf,
            &mut self.list_state,
        );
    }
}
