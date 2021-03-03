pub mod explorer;
pub mod watchlist;

use std::collections::BTreeMap;

use explorer::ViewExplorerHome;
use watchlist::ViewWatchList;

// use tui-tree-widget::

use crate::tango_utils::TangoDevicesLookup;
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

type DeviceName = String;
type AttributeName = String;
type AttributeValue = Option<String>;
#[derive(Debug)]
pub enum TabChoice {
    Explorer,
    WatchList,
}

impl Default for TabChoice {
    fn default() -> TabChoice {
        TabChoice::Explorer
    }
}

#[derive(Debug, Default)]
pub struct SharedViewState {
    pub tango_host: Option<String>,
    pub current_selected_device: Option<String>,
    pub watch_list: BTreeMap<DeviceName, BTreeMap<AttributeName, AttributeValue>>,
    pub current_tab: TabChoice,
}

impl SharedViewState {
    pub fn add_watch_attribute(&mut self, attribute_name: String) {
        if let Some(current_device) = &self.current_selected_device {
            // Add the device if not present
            self.watch_list
                .entry(current_device.clone())
                .or_insert(BTreeMap::default());
            // Add the attribute if not present
            if let Some(attr_map) = self.watch_list.get_mut(current_device) {
                attr_map.entry(attribute_name).or_insert(None);
            }
        };
    }

    pub fn _remove_watch_attribute(&mut self, attribute_name: String) {
        if let Some(current_device) = &self.current_selected_device {
            if let Some(attr_map) = self.watch_list.get_mut(current_device) {
                attr_map.remove(&attribute_name);
            }

            if let Some(attr_map) = self.watch_list.get(current_device) {
                if attr_map.len() == 0 {
                    self.watch_list.remove(current_device);
                }
            }
        }
    }
}

// #[derive(Debug)]
pub enum View<'a> {
    Explorer(ViewExplorerHome<'a>),
    WatchList(ViewWatchList),
}

// #[derive(Debug, Default)]

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
        let tango_host_text = Paragraph::new(format!(
            "TANGO_HOST: {}",
            shared_view_state
                .tango_host
                .as_ref()
                .unwrap_or(&String::from(""))
        ))
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(Block::default().style(Style::default().fg(Color::White)));
        f.render_widget(tango_host_text, area);
    }

    fn draw_menu<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        let mut menu_items: Vec<MenuOption> = self.get_default_menu_items().clone();
        menu_items.extend(self.get_view_menu_items(shared_view_state).clone());
        let mut rows: Vec<Row> = menu_items
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
        rows.insert(0, Row::new(vec![Cell::from("")]));

        let table = Table::new(rows)
            // You can set the style of the entire Table.
            .style(Style::default().fg(Color::White))
            // It has an optional header, which is simply a Row always visible at the top.
            // As any other widget, a Table can be wrapped in a Block.
            // Columns widths are constrained in the same way as Layout...
            .widths(&[
                Constraint::Length(10),
                Constraint::Length(15),
                Constraint::Length(15),
            ])
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1);
        f.render_widget(table, area);
    }

    fn draw_tabs<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        let selected_tab = match shared_view_state.current_tab {
            TabChoice::Explorer => 0,
            TabChoice::WatchList => 1,
        };
        let tab_titles = ["Explorer", "Watchlist"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();
        let tabs = Tabs::new(tab_titles)
            .select(selected_tab)
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

    fn draw_watchlist<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let length_left = area.width / 3;
        let length_right = area.width - length_left;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(length_left),
                    Constraint::Length(length_right),
                ]
                .as_ref(),
            )
            .split(area);

        let left = Paragraph::new("Left  W")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );

        let right = Paragraph::new("Right  W")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );
        f.render_widget(left, chunks[0]);
        f.render_widget(right, chunks[1]);
    }

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
        _tango_devices_lookup: &TangoDevicesLookup,
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

    fn draw<B: Backend>(&self, f: &mut Frame<B>, shared_view_state: &mut SharedViewState) {
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
                    Constraint::Length(3), // Messages
                ]
                .as_ref(),
            )
            .split(size);

        self.draw_header(f, chunks[0], shared_view_state);
        self.draw_menu(f, chunks[1], shared_view_state);
        self.draw_tabs(f, chunks[2], shared_view_state);
        self.draw_body(f, chunks[3], shared_view_state);
        self.draw_footer(f, chunks[4]);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_view() {
//         let v = View::default();
//         println!("{:#?}", v);
//         match v {
//             View::ExplorerHome(a) => a.test_draw(),
//             View::ExplorerCommands(a) => a.test_draw(),
//             View::ExplorerAttributes(a) => a.test_draw(),
//             View::WatchList(a) => a.test_draw(),
//         }
//     }
// }
