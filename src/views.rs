pub mod command;
pub mod confirm_command;
pub mod explorer;
pub mod watchlist;

use command::ViewCommand;
use confirm_command::ViewConfirmCommand;
use explorer::ViewExplorerHome;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{mpsc, Arc, Mutex};
use watchlist::ViewWatchList;

use crate::tango_utils::TangoDevicesLookup;
use crate::views::watchlist::AttributeReading;
use crate::Event;
use crossterm::event::KeyEvent;
use std::hash::Hash;
use tui::symbols::line::DOUBLE_VERTICAL;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

use self::command::ExecutedCommands;
pub type DeviceName = String;
pub type AttributeName = String;
pub type AttributeReadings = BTreeMap<DeviceName, BTreeMap<AttributeName, AttributeReading>>;

// The SharedViewState is information that are shared between the different tabs
// and sections within the tab itself.
#[derive(Debug)]
pub struct SharedViewState<'a> {
    pub tango_host: Option<String>,
    pub selected_device: Option<String>,
    pub watch_list: Arc<Mutex<AttributeReadings>>,
    pub current_view: View,
    pub tango_devices_lookup: TangoDevicesLookup<'a>,
    pub executed_commands: ExecutedCommands,
}

impl SharedViewState<'_> {
    pub fn new(tx_commands: mpsc::Sender<Event>) -> Self {
        Self {
            tango_host: None,
            selected_device: None,
            watch_list: Arc::default(),
            current_view: View::Explorer,
            tango_devices_lookup: TangoDevicesLookup::default(),
            executed_commands: ExecutedCommands::new(tx_commands),
        }
    }

    pub fn add_watch_attribute(&mut self, attribute_name: String) {
        if let Some(current_device) = &self.selected_device {
            // Add the device if not present
            self.watch_list
                .lock()
                .unwrap()
                .entry(current_device.clone())
                .or_insert_with(BTreeMap::default);
            // Add the attribute if not present
            if let Some(attr_map) = self.watch_list.lock().unwrap().get_mut(current_device) {
                attr_map
                    .entry(attribute_name)
                    .or_insert_with(AttributeReading::default);
            }
        };
    }

    pub fn _remove_watch_attribute(&mut self, attribute_name: String) {
        if let Some(current_device) = &self.selected_device {
            if let Some(attr_map) = self.watch_list.lock().unwrap().get_mut(current_device) {
                attr_map.remove(&attribute_name);
            }

            if let Some(attr_map) = self.watch_list.lock().unwrap().get(current_device) {
                if attr_map.is_empty() {
                    self.watch_list.lock().unwrap().remove(current_device);
                }
            }
        }
    }
    pub fn toggle_current_view(&mut self) {
        match self.current_view {
            View::Command => self.current_view = View::Explorer,
            View::WatchList => self.current_view = View::Command,
            View::Explorer => self.current_view = View::WatchList,
            View::ConfirmCommand => self.current_view = View::Command,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum View {
    Command,
    ConfirmCommand,
    WatchList,
    Explorer,
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            View::Command => write!(f, "Command"),
            View::ConfirmCommand => write!(f, "ConfirmCommand"),
            View::WatchList => write!(f, "WatchList"),
            View::Explorer => write!(f, "Explorer"),
        }
    }
}

// #[derive(Debug)]
pub enum ViewType<'a> {
    Explorer(ViewExplorerHome<'a>),
    WatchList(ViewWatchList),
    Command(ViewCommand),
    ConfirmCommand(ViewConfirmCommand),
}

// The views are stored in a hashmap.
// The key is `View.to_string()` and the value is the `View`
impl<'a> fmt::Display for ViewType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ViewType::Explorer(_) => write!(f, "Explorer"),
            ViewType::WatchList(_) => write!(f, "Watchlist"),
            ViewType::Command(_) => write!(f, "Command"),
            ViewType::ConfirmCommand(_) => write!(f, "Popup"),
        }
    }
}

impl From<&ViewType<'_>> for usize {
    fn from(item: &ViewType) -> Self {
        match item {
            ViewType::Explorer(_) => 0,
            ViewType::WatchList(_) => 1,
            ViewType::Command(_) => 2,
            ViewType::ConfirmCommand(_) => 3,
        }
    }
}

impl From<ViewType<'_>> for View {
    fn from(val: ViewType<'_>) -> Self {
        match val {
            ViewType::Explorer(_) => View::Explorer,
            ViewType::WatchList(_) => View::WatchList,
            ViewType::Command(_) => View::Command,
            ViewType::ConfirmCommand(_) => View::ConfirmCommand,
        }
    }
}

impl From<&ViewType<'_>> for View {
    fn from(val: &ViewType<'_>) -> Self {
        match val {
            ViewType::Explorer(_) => View::Explorer,
            ViewType::WatchList(_) => View::WatchList,
            ViewType::Command(_) => View::Command,
            ViewType::ConfirmCommand(_) => View::ConfirmCommand,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct MenuOption {
    key: String,
    description: String,
}

pub trait Draw {
    fn get_default_menu_items(&self) -> Vec<MenuOption> {
        vec![
            MenuOption {
                key: "ESC".to_string(),
                description: "Quit".to_string(),
            },
            MenuOption {
                key: "TAB".to_string(),
                description: "Toggle Tabs".to_string(),
            },
        ]
    }

    fn get_view_menu_items(&self, _shared_view_state: &mut SharedViewState) -> Vec<MenuOption> {
        vec![]
    }

    fn draw_header<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(area.width / 2),
                    Constraint::Length(area.width / 2),
                ]
                .as_ref(),
            )
            .split(area);

        let tango_host_text =
            Paragraph::new("Ｔａｎｇｏ  Ｃｏｎｔｒｏｌｓ  Ｅｘｐｌｏｒｅｒ  ＴＵＩ")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Left);
        // .block(Block::default().style(Style::default().fg(Color::White)));
        f.render_widget(tango_host_text, chunks[0]);

        let program_name_text = Paragraph::new(format!(
            "TANGO_HOST: {}",
            shared_view_state
                .tango_host
                .as_ref()
                .unwrap_or(&String::from(""))
        ))
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Right);
        // .block(Block::default().style(Style::default().fg(Color::White)));
        f.render_widget(program_name_text, chunks[1]);
    }

    fn draw_menu<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        let mut menu_items: Vec<MenuOption> = self.get_default_menu_items();
        menu_items.extend(self.get_view_menu_items(shared_view_state));

        // Split menu left/right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(area.width / 3),
                    Constraint::Length(area.width / 3),
                    Constraint::Length(area.width / 3),
                ]
                .as_ref(),
            )
            .split(area);

        let rows: Vec<Row> = menu_items
            .into_iter()
            .map(|menu_option| {
                Row::new(vec![
                    Cell::from(Spans::from(vec![
                        Span::styled("<", Style::default().fg(Color::LightCyan)),
                        Span::styled(menu_option.key, Style::default().fg(Color::White)),
                        Span::styled(">", Style::default().fg(Color::LightCyan)),
                    ])),
                    Cell::from(menu_option.description).style(Style::default().fg(Color::White)),
                ])
            })
            .collect();

        let left_rows: Vec<Row> = rows.iter().take(3).cloned().collect();
        let left_table = Table::new(left_rows)
            .style(Style::default().fg(Color::White))
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(15),
                Constraint::Length(15),
            ])
            .column_spacing(1);
        f.render_widget(left_table, chunks[0]);

        let right_rows: Vec<Row> = rows.iter().skip(3).cloned().collect();
        let right_table = Table::new(right_rows)
            .style(Style::default().fg(Color::White))
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(15),
                Constraint::Length(15),
            ])
            .column_spacing(1);
        f.render_widget(right_table, chunks[1]);
    }

    fn draw_tabs<B: Backend>(&self, f: &mut Frame<B>, area: Rect, tab_index: usize) {
        let tab_titles = ["Explorer", "Watchlist", "Command"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();
        let tabs = Tabs::new(tab_titles)
            .select(tab_index)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .divider(DOUBLE_VERTICAL);
        f.render_widget(tabs, area);
    }

    fn draw_explorer<B: Backend>(
        &self,
        _f: &mut Frame<B>,
        _area: Rect,
        _shared_view_state: &mut SharedViewState,
    ) {
    }

    fn draw_watchlist<B: Backend>(&self, _f: &mut Frame<B>, _area: Rect) {}

    fn draw_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let footer = Paragraph::new("Tango Controls Explorer TUI")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );
        f.render_widget(footer, area);
    }

    fn handle_event(
        &mut self,
        _key_event: &KeyEvent,
        _shared_view_state: &mut SharedViewState,
    ) -> usize {
        0
    }

    fn draw_body<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        self.draw_explorer(f, area, shared_view_state);
    }

    fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        shared_view_state: &mut SharedViewState,
        tab_index: usize,
    ) {
        let size = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(1), // TANGO HOST
                    Constraint::Length(6), // Instructions
                    Constraint::Length(3), // Tabs
                    Constraint::Min(2),    // Explorer
                                           // Constraint::Length(3), // Messages
                ]
                .as_ref(),
            )
            .split(size);

        self.draw_header(f, chunks[0], shared_view_state);
        self.draw_menu(f, chunks[1], shared_view_state);
        self.draw_tabs(f, chunks[2], tab_index);
        self.draw_body(f, chunks[3], shared_view_state);
        // self.draw_footer(f, chunks[4]);
    }
}
