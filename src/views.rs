pub mod explorer;
pub mod watchlist;

use explorer::ViewExplorerHome;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use watchlist::ViewWatchList;

use crate::tango_utils::TangoDevicesLookup;
use crate::views::watchlist::AttributeReading;
use crossterm::event::KeyEvent;
use tui::symbols::line::DOUBLE_VERTICAL;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub type DeviceName = String;
pub type AttributeName = String;

pub type AttributeReadings = BTreeMap<DeviceName, BTreeMap<AttributeName, AttributeReading>>;

// The SharedViewState is information that are shared between the different tabs
// and sections within the tab itself.
#[derive(Debug, Default)]
pub struct SharedViewState<'a> {
    pub tango_host: Option<String>,
    pub current_selected_device: Option<String>,
    pub watch_list: Arc<Mutex<AttributeReadings>>,
    pub current_view: String,
    pub tango_devices_lookup: TangoDevicesLookup<'a>,
}

impl SharedViewState<'_> {
    pub fn add_watch_attribute(&mut self, attribute_name: String) {
        if let Some(current_device) = &self.current_selected_device {
            // Add the device if not present
            self.watch_list
                .lock()
                .unwrap()
                .entry(current_device.clone())
                .or_insert(BTreeMap::default());
            // Add the attribute if not present
            if let Some(attr_map) = self.watch_list.lock().unwrap().get_mut(current_device) {
                attr_map
                    .entry(attribute_name)
                    .or_insert(AttributeReading::default());
            }
        };
    }

    pub fn _remove_watch_attribute(&mut self, attribute_name: String) {
        if let Some(current_device) = &self.current_selected_device {
            if let Some(attr_map) = self.watch_list.lock().unwrap().get_mut(current_device) {
                attr_map.remove(&attribute_name);
            }

            if let Some(attr_map) = self.watch_list.lock().unwrap().get(current_device) {
                if attr_map.len() == 0 {
                    self.watch_list.lock().unwrap().remove(current_device);
                }
            }
        }
    }
    pub fn toggle_current_view(&mut self) {
        match self.current_view.as_str() {
            "Explorer" => self.current_view = String::from("Watchlist"),
            "Watchlist" => self.current_view = String::from("Explorer"),
            _ => panic!("Tab position for {} not defined", self.current_view),
        };
    }
}

// #[derive(Debug)]
pub enum View<'a> {
    Explorer(ViewExplorerHome<'a>),
    WatchList(ViewWatchList),
}

// The views are stored in a hashmap.
// The key is `View.to_string()` and the value is the `View`
impl<'a> fmt::Display for View<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            View::Explorer(_) => {
                write!(f, "Explorer")
            }
            View::WatchList(_) => {
                write!(f, "Watchlist")
            }
        }
    }
}

// The Into here is used to translate a View into an usize.
// The usize is used to determine which tab to select
impl Into<usize> for &View<'_> {
    fn into(self) -> usize {
        match self {
            View::Explorer(_) => 0,
            View::WatchList(_) => 1,
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

        let tango_host_text = Paragraph::new("Tango Controls Explorer TUI")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Left);
        // .block(Block::default().style(Style::default().fg(Color::White)));
        f.render_widget(tango_host_text, chunks[0]);

        let program_name_text = Paragraph::new(format!(
            "  TANGO_HOST: {}",
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
        let mut menu_items: Vec<MenuOption> = self.get_default_menu_items().clone();
        menu_items.extend(self.get_view_menu_items(shared_view_state).clone());

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
        let tab_titles = ["Explorer", "Watchlist"]
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
