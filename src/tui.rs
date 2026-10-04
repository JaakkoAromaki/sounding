use std::{fs,io};
use std::path::{PathBuf, Path};

// own
use crate::player::Player;
use crate::config::Config;
use crate::utils::find_filename;

use std::time::Duration;
use ratatui::widgets::{Borders, Clear, Paragraph, Wrap};
use tui_big_text::{BigText, PixelSize};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect, Alignment, Flex},
    style::{Style, Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        Block, List, ListItem, ListState, StatefulWidget, Widget
    },
    DefaultTerminal, Frame,
};

#[derive(Debug)]
pub struct App {
    exit: bool,
    list_state: ListState,
    items: Vec<PathBuf>,
    player: Player,

    current_chosen: Option<PathBuf>,
    songs_list_open: bool,
    current_song: String,
    current_dir: PathBuf,
    popup_on: bool,
    input: String,
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

        let current_chosen = selected
            .and_then(|index| items.get(index).cloned());
        // luo player
        let mut player = Player::new()?;

        let current_song = player
                                    .get_playing()?
                                    .unwrap_or_default();

        let current_song = find_filename(Path::new(&current_song));

        // init
        Ok(Self {
            exit: false,
            list_state: ListState::default().with_selected(selected),
            items,
            player,

            current_chosen,
            songs_list_open: true,
            current_song,
            current_dir: PathBuf::from(&config.music_directory),
            popup_on: false,
            input: String::new(),
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn load_directory(&mut self, path: &Path) {
    self.items = fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<PathBuf>>()
        })
        .unwrap_or_default();

    self.current_dir = path.to_path_buf();

    self.list_state.select(if self.items.is_empty() {
        None
    } else {
        Some(0)
    });
}

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut *self, frame.area());
    }


    fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(());
        }

        let Event::Key(key) = event::read()? else {
            return Ok(());
        };

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        
        if self.popup_on {
            match key.code {
                KeyCode::Esc => {
                    self.popup_on = false;
                }

                KeyCode::Backspace => {
                    self.input.pop();
                }

                KeyCode::Char(c) => {
                    self.input.push(c);
                }

                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Enter => {
                if let Some(index) = self.list_state.selected() {
                    if let Some(selected_item) = self.items.get(index).cloned() {
                        if selected_item.is_dir() {
                            self.load_directory(&selected_item);
                        } else {
                            self.current_song = find_filename(&selected_item);
                            self.current_chosen = Some(selected_item.clone());

                            self.player.play(&selected_item)?;
                            let _ = self.player.depause();
                        }
                    }
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

            KeyCode::Char(' ') => {
                let _ = self.player.toggle_pause_play();
            }

            KeyCode::Char('p') => {
                let _ = self.player.depause();
            }

            KeyCode::Tab => {
                self.songs_list_open = !self.songs_list_open;
            }

            KeyCode::Char('a') => {
                self.popup_on = true;
            }

            KeyCode::Backspace => {
                if let Some(parent) =
                    self.current_dir.parent().map(Path::to_path_buf)
                {
                    self.load_directory(&parent);
                }
            }

            _ => {}
        }

        Ok(())
    }
}


// ui

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

        Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
    }

fn format_time(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;

    format!("{}:{:02}", minutes, seconds)
}
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (constraint1, constraint2) = if self.songs_list_open {
            (25, 75)
        } else {
            (0, 100)
        };

        let mainlayout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(constraint1),
                Constraint::Percentage(constraint2),
            ])
            .spacing(1)
            .split(area);

        let items = self
            .items
            .iter()
            .map(|path| {
                let title = find_filename(path);

                let mut item = ListItem::new(title);

                if self.current_chosen.as_ref() == Some(path) {
                    item = item.style(
                        Style::default().fg(Color::Green)
                    );
                }

                item
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(Line::from(" Songs ".bold()).centered())
                    .border_set(border::PLAIN),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White),
            )
            .highlight_symbol("> ");

        if self.songs_list_open {
            StatefulWidget::render(
                list,
                mainlayout[0],
                buf,
                &mut self.list_state,
            );
        }

        let right_area = mainlayout[1];

        let now_playing = Block::bordered()
            .title("")
            .border_set(border::EMPTY);

        let inner = now_playing.inner(right_area);

        let current_song = if self.current_song.is_empty() {
            ""
        } else {
            &self.current_song
        };

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Octant)
            .style(Style::default().blue())
            .lines(vec![
                current_song.red().into(),
            ])
            .alignment(Alignment::Center)
            .build();

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .flex(Flex::Center)
            .split(inner);

        now_playing.render(right_area, buf);

        big_text.render(areas[1], buf);

        let current_seconds = self.player
            .get_progress()
            .ok()
            .flatten()
            .unwrap_or(0.0) as u64;
        let total_seconds = self.player
            .get_duration()
            .ok()
            .flatten()
            .unwrap_or(0.0) as u64;

        let progress_text = Paragraph::new(format!(
            "{} / {}",
            format_time(current_seconds),
            format_time(total_seconds)
        ))
        .alignment(Alignment::Center);

        progress_text.render(areas[2], buf);

        if self.popup_on {
            let popup_area = centered_rect(60, 30, area);

            Clear.render(popup_area, buf);

            let popup = Block::bordered()
                .title(" Input ")
                .borders(Borders::ALL);

            popup.render(popup_area, buf);

            let inner = popup_area.inner(ratatui::layout::Margin {
                horizontal: 2,
                vertical: 1,
            });

            let input = Paragraph::new(self.input.as_str())
                .block(
                    Block::bordered()
                        .title(" Search ")
                        .borders(Borders::NONE),
                );

            input.render(inner, buf);
        }
    }
}