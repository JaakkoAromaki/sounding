use std::{fs,io};
use std::path::{PathBuf, Path};

// own
use crate::player::Player;
use crate::youtube::{download, search};
use crate::config::Config;
use crate::utils::find_filename;

// ui bs
use std::time::Duration;
use ratatui::widgets::{Borders, Clear, Paragraph};
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
use yt_dlp::model::playlist::PlaylistEntry;


// todo: puhdista tää vitun class
// ja puhdista variable nimet ( ne on bs )
// - hamlak


#[derive(Debug)]
pub struct App {
    exit: bool,
    list_state: ListState,
    search_results_list_state: ListState,
    queue_list_state: ListState,
    items: Vec<PathBuf>,
    queue: Vec<PathBuf>,
    queue_playing: bool,
    player: Player,

    current_chosen: Option<PathBuf>,
    songs_list_open: bool,
    current_song: String,
    current_dir: PathBuf,
    popup_on: bool,
    input: String,
    search_results: Vec<PlaylistEntry>,
    notification: Option<(String, std::time::Instant)>,
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
        let current_song = player.get_playing()?.unwrap_or_default();
        let current_song = find_filename(Path::new(&current_song));

        // init
        Ok(Self {
            exit: false,
            list_state: ListState::default().with_selected(selected),
            search_results_list_state: ListState::default(),
            queue_list_state: ListState::default(),
            items,
            queue: Self::load_queue(),
            queue_playing: false,
            player,

            current_chosen,
            songs_list_open: true,
            current_song,
            current_dir: PathBuf::from(&config.music_directory),
            popup_on: false,
            input: String::new(),
            search_results: Vec::new(),
            notification: None,
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            self.check_queue()?;
            self.handle_events().await?;
        }

        Ok(())
    }

    fn play_queue_item(&mut self) -> io::Result<()> {
        if let Some(path) = self.queue.first().cloned() {
            self.current_song = find_filename(&path);
            self.current_chosen = Some(path.clone());

            self.player.play(&path)?;
            let _ = self.player.depause();
        }

        Ok(())
    }
    fn check_queue(&mut self) -> io::Result<()> {
        if !self.queue_playing || self.queue.is_empty() {
            return Ok(());
        }

        let progress = self.player
            .get_progress()
            .ok()
            .flatten()
            .unwrap_or(0.0);

        let duration = self.player
            .get_duration()
            .ok()
            .flatten()
            .unwrap_or(0.0);

        if duration > 0.0 && progress >= duration {
            self.queue.remove(0);
            self.save_queue()?;

            if !self.queue.is_empty() {
                self.play_queue_item()?;
            } else {
                self.queue_playing = false;
            }
        }

        Ok(())
    }
    fn save_queue(&self) -> io::Result<()> {
        let paths: Vec<String> = self
            .queue
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

        let json = serde_json::to_string_pretty(&paths)
            .map_err(io::Error::other)?;

        fs::write("queue.json", json)?;

        Ok(())
    }

    fn load_queue() -> Vec<PathBuf> {
        let Ok(data) = fs::read_to_string("queue.json") else {
            return Vec::new();
        };

        let Ok(paths) = serde_json::from_str::<Vec<String>>(&data) else {
            return Vec::new();
        };

        paths.into_iter()
            .map(PathBuf::from)
            .filter(|path| path.exists())
            .collect()
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

        // lataa index
        self.list_state.select(if self.items.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&mut *self, frame.area());
    }


    async fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(());
        }

        let Event::Key(key) = event::read()? else {
            return Ok(());
        };

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        
        // search popup
        if self.popup_on {
            match key.code {
                KeyCode::Up => {
                    if !self.search_results.is_empty() {
                        self.search_results_list_state.select_previous();
                    }
                }

                KeyCode::Down => {
                    if !self.search_results.is_empty() {
                        self.search_results_list_state.select_next();
                    }
                }
                KeyCode::Tab => {
                    if let Some(index) = self.search_results_list_state.selected() {
                        if let Some(entry) = self.search_results.get(index) {
                            let name = entry.title
                                .chars()
                                .map(|c| {
                                    if ['/', '\\', ':', '*', '?', '"', '<', '>', '|'].contains(&c) {
                                        '_'
                                    } else {
                                        c
                                    }
                                })
                                .collect::<String>();

                            match download(
                                &entry.url,
                                &self.current_dir,
                                &name,
                            ).await {
                                Ok(_) => {
                                    self.notification = Some((
                                        format!("Downloaded {}", name),
                                        std::time::Instant::now(),
                                    ));
                                }
                                Err(_e) => {
                                }
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    self.popup_on = false;
                }

                KeyCode::Backspace => {
                    self.input.pop();
                }

                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Enter => {
                    let query = self.input.clone();

                    match search(&query).await {
                        Ok(results) => {
                            self.search_results = results.entries;

                            self.search_results_list_state.select(
                                if self.search_results.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                }
                            );
                        }
                        Err(_e) => {
                            eprintln!("Search failed: {_e}");
                        }
                    }
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
                            self.queue_playing = false;

                            self.player.play(&selected_item)?;
                            let _ = self.player.depause();
                        }
                    }
                }
            }
            KeyCode::Char('m') => {
                if !self.queue_playing {
                    self.queue_playing = true;
                    self.play_queue_item()?;
                }
            }
            KeyCode::Char('n') => {
                if let Some(index) = self.list_state.selected() {
                    if let Some(selected_item) = self.items.get(index).cloned() {
                        if selected_item.is_dir() {
                            self.load_directory(&selected_item);
                        } else {
                            self.queue.push(selected_item);
                            self.save_queue()?;
                        }
                    }
                }
            }
            KeyCode::Char('s') => {
                if self.queue_playing && !self.queue.is_empty() {
                    self.queue.remove(0);
                    self.save_queue()?;

                    if self.queue.is_empty() {
                        self.queue_playing = false;
                        let _ = self.player.toggle_pause_play();
                        self.notification = Some((
                            format!("queue is empty"),
                            std::time::Instant::now(),
                        ));
                    } else {
                        self.play_queue_item()?;
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
                self.queue_playing = false;
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
fn bottom_center_area(area: Rect, width: u16, height: u16) -> Rect {
    // 1. Center horizontally by putting the target width in the middle of excess space
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(area);

    // 2. Align vertically to the bottom of that horizontal slice
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(height)])
        .flex(Flex::End)
        .split(horizontal_layout[0]);

    vertical_layout[0]
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
        let sidelayout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .spacing(1)
            .split(mainlayout[0]);

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

        let queue_items = self
            .queue
            .iter()
            .map(|path| {
                let title = find_filename(path);
                ListItem::new(title)
            })
            .collect::<Vec<_>>();

        let queue = List::new(queue_items)
            .block(
                Block::bordered()
                    .title(Line::from(" Queue ".bold()).centered())
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
                &list,
                sidelayout[0],
                buf,
                &mut self.list_state,
            );
        }
        if self.songs_list_open {
            StatefulWidget::render(
                &queue,
                sidelayout[1],
                buf,
                &mut self.queue_list_state,
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
            let popup_area = centered_rect(70, 60, area);

            Clear.render(popup_area, buf);

            let popup = Block::bordered()
                .title(" Search ")
                .borders(Borders::ALL);

            popup.render(popup_area, buf);

            let inner = popup_area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });

            let popup_layout = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(inner);

            let input = Paragraph::new(self.input.as_str())
                .block(
                    Block::bordered()
                        .title(" Query ")
                        .borders(Borders::ALL),
                );

            input.render(popup_layout[0], buf);

            let result_items = self
                .search_results
                .iter()
                .map(|entry| {
                    ListItem::new(
                        entry.title.clone()
                    )
                })
                .collect::<Vec<_>>();

            let results = List::new(result_items)
                .block(
                    Block::bordered()
                        .title(" Results (TAB to download)")
                        .borders(Borders::NONE),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White),
                )
                .highlight_symbol("> ");

            StatefulWidget::render(results, popup_layout[1], buf, &mut self.search_results_list_state);
        }
        // notification
        if let Some((message, time)) = &self.notification {
            if time.elapsed() < Duration::from_secs(3) {
                let size = area;
                let text_area = bottom_center_area(size, 80, 3);

                let text = vec![
                    Line::from(message.as_str()).centered(),
                ];

                let paragraph = Paragraph::new(text)
                    .block(Block::bordered().title("Notification"))
                    .style(Style::default().fg(Color::Yellow));

                paragraph.render(text_area, buf);
            } else { self.notification = None }
        }
    }
}