use crate::tango_utils;
use crate::views::{Draw, SharedViewState};
use log::error;
use std::convert::From;
use tui::{
    backend::Backend,
    layout::Constraint,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Row, Table, TableState},
    Frame,
};

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
                Ok(attr_data_option) => {
                    match attr_data_option {
                        Some(attr_data) => {
                            match attr_data.data.into_string() {
                                Ok(val) => *self = AttributeReading::Value(val),
                                // Looks like err is a valid value ?
                                Err(err) => *self = AttributeReading::Value(format!("{}", err)),
                            }
                        }
                        None => {
                            *self = AttributeReading::Value("Error reading attribute".to_string());
                        }
                    }
                }
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

    fn draw_table<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
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

        let table = Table::new(table_items)
            // You can set the style of the entire Table.
            .style(Style::default().fg(Color::White))
            // It has an optional header, which is simply a Row always visible at the top.
            .header(
                Row::new(header)
                    .style(Style::default().fg(Color::LightCyan))
                    // If you want some space between the header and the rest of the rows, you can always
                    // specify some margin at the bottom.
                    .bottom_margin(1),
            )
            // As any other widget, a Table can be wrapped in a Block.
            .block(Block::default().title(""))
            // Columns widths are constrained in the same way as Layout...
            .widths(&widths)
            // ...and they can be separated by a fixed spacing.
            .column_spacing(1)
            // If you wish to highlight a row in any specific way when it is selected...
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            // ...and potentially show a symbol in front of the selection.
            .highlight_symbol(">>");

        f.render_stateful_widget(table, area, &mut self.stateful_table.clone());
    }
}

impl Draw for ViewWatchList {
    fn draw_body<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        self.draw_table(f, area, shared_view_state);
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
