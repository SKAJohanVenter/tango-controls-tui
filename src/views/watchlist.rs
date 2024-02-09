use crate::tango_utils;
use crate::views::{Draw, SharedViewState};
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{
    layout::Constraint,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Row, Table, TableState},
    Frame,
};
use std::convert::From;

use super::MenuOption;

#[derive(Debug, Clone)]
pub enum AttributeReading {
    Value(String),
    Error(String),
}

impl Default for AttributeReading {
    fn default() -> AttributeReading {
        AttributeReading::Value(String::from(""))
    }
}

impl AttributeReading {
    pub fn update(&mut self, device_name: &str, attr_name: &str) -> &mut AttributeReading {
        if let AttributeReading::Value(_) = self {
            match tango_utils::read_attribute(device_name, attr_name) {
                Ok(attr_data_option) => match attr_data_option {
                    // Some(attr_data) => *self = AttributeReading::Value(attr_data.data.to_string()),
                    Some(attr_data) => {
                        *self = AttributeReading::Value(format!("{}", attr_data.data))
                    }
                    None => {
                        *self = AttributeReading::Value("Error reading attribute".to_string());
                    }
                },
                Err(err) => {
                    *self = AttributeReading::Error("Reading of type unsupported".to_string());
                    error!(
                        "Reading conversion error for {}/{}: {}",
                        device_name, attr_name, err
                    );
                }
            };
        };
        self
    }
}

#[derive(Default, Debug)]
pub struct ViewWatchList {
    stateful_table: TableState,
}

impl ViewWatchList {
    pub fn new() -> ViewWatchList {
        ViewWatchList {
            stateful_table: TableState::default(),
        }
    }

    fn draw_table(&self, f: &mut Frame, area: Rect, shared_view_state: &mut SharedViewState) {
        let header = vec!["Device", "Attribute", "Value"];
        let widths = {
            let size_a = area.width / 6;
            let size_b = area.width / 6;
            let size_c = area.width - size_a - size_b;
            vec![
                Constraint::Length(size_a),
                Constraint::Length(size_b),
                Constraint::Length(size_c),
            ]
        };

        let mut table_items: Vec<Row> = Vec::new();
        let watch_l = &shared_view_state.watch_list.lock().unwrap();
        for (device_name, attr_map) in watch_l.iter() {
            for (attr_name, attr_value) in attr_map {
                let attr_reading = match attr_value {
                    AttributeReading::Value(val) => val,
                    AttributeReading::Error(val) => val,
                };
                table_items.push(Row::new(vec![
                    device_name.clone(),
                    attr_name.clone(),
                    attr_reading.clone(),
                ]));
            }
        }

        let table = Table::new(table_items, &widths)
            .style(Style::default().fg(Color::White))
            .header(
                Row::new(header)
                    .style(Style::default().fg(Color::LightCyan))
                    .bottom_margin(1),
            )
            .block(Block::default().title(""))
            .column_spacing(1)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">>");

        f.render_stateful_widget(table, area, &mut self.stateful_table.clone());
    }
}

impl Draw for ViewWatchList {
    fn get_view_menu_items(&self, _shared_view_state: &mut SharedViewState) -> Vec<MenuOption> {
        let items = vec![MenuOption {
            key: "c".to_string(),
            description: "Clear list".to_string(),
        }];
        items
    }

    fn draw_body(&self, f: &mut Frame, area: Rect, shared_view_state: &mut SharedViewState) {
        self.draw_table(f, area, shared_view_state);
    }

    fn handle_event(
        &mut self,
        key_event: &KeyEvent,
        shared_view_state: &mut SharedViewState,
    ) -> usize {
        if let KeyCode::Char('c') = key_event.code {
            let watch_l = &mut shared_view_state.watch_list.lock().unwrap();
            watch_l.clear();
        }
        0
    }
}

impl From<usize> for ViewWatchList {
    fn from(_item: usize) -> Self {
        ViewWatchList::new()
    }
}

impl From<ViewWatchList> for usize {
    fn from(_item: ViewWatchList) -> usize {
        1
    }
}
