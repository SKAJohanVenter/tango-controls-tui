use crate::stateful_tree::StatefulTree;
use crate::tango_utils::TangoDevicesLookup;
use crate::tango_utils::{
    display_attribute_format, display_attribute_type, get_attribute_list, get_command_list,
    GetTreeItems,
};
use crate::views::{Draw, MenuOption, SharedViewState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};
use ratatui_tree_widget::Tree;
use std::convert::From;
use tango_controls_client_sys::types::CmdArgType;

use super::View;

#[derive(PartialEq)]
enum Focus {
    Left,
    Right,
}
#[derive(PartialEq)]
enum DeviceDisplay {
    Commands,
    Attributes,
    Empty,
}

#[derive(Default, Clone)]
pub struct RowId {
    name: String,
    in_type: Option<CmdArgType>,
}

pub struct ViewExplorerHome<'a> {
    stateful_tree: StatefulTree<'a>,
    focus: Focus,
    stateful_table: TableState,
    stateful_table_items: Vec<(RowId, Row<'a>)>,
    device_display: DeviceDisplay,
}

impl<'a> ViewExplorerHome<'a> {
    pub fn new(tdl: &TangoDevicesLookup<'a>) -> ViewExplorerHome<'a> {
        ViewExplorerHome {
            stateful_tree: StatefulTree::with_items(tdl.get_tree_items()),
            focus: Focus::Left,
            stateful_table: TableState::default(),
            stateful_table_items: Vec::new(),
            device_display: DeviceDisplay::Empty,
        }
    }

    fn draw_left<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        _shared_view_state: &mut SharedViewState,
    ) {
        let mut items = Tree::new(self.stateful_tree.items.to_vec())
            .block(Block::default().borders(Borders::ALL).title("Device Tree"));
        if self.focus == Focus::Left {
            items = items
                .highlight_style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
        }

        f.render_stateful_widget(items, area, &mut self.stateful_tree.state.clone());
    }

    fn populate_device_items(
        &mut self,
        shared_view_state: &SharedViewState,
        device_display: DeviceDisplay,
    ) {
        self.stateful_table_items.clear();
        match device_display {
            DeviceDisplay::Commands => {
                if let Some(current_device) = shared_view_state.selected_device.clone() {
                    match get_command_list(current_device.as_str()) {
                        Ok(commands) => {
                            for comm in commands {
                                self.stateful_table_items.push((
                                    RowId {
                                        name: comm.cmd_name.clone(),
                                        in_type: Some(comm.in_type),
                                    },
                                    Row::new(vec![
                                        comm.cmd_name,
                                        format!("{:?}", comm.in_type),
                                        format!("{:?}", comm.out_type),
                                    ]),
                                ));
                            }
                        }
                        Err(err) => {
                            self.stateful_table_items.push((
                                RowId::default(),
                                Row::new(vec![
                                    format!("Error retrieving info: {}", err),
                                    "".to_string(),
                                ]),
                            ));
                        }
                    }
                }
            }
            DeviceDisplay::Attributes => {
                if let Some(current_device) = shared_view_state.selected_device.clone() {
                    match get_attribute_list(current_device.as_str()) {
                        Ok(attributes) => {
                            for attr in attributes {
                                self.stateful_table_items.push((
                                    RowId {
                                        name: attr.attribute_info.name.to_string(),
                                        in_type: None,
                                    },
                                    Row::new(vec![
                                        attr.attribute_info.name,
                                        display_attribute_type(attr.attribute_data),
                                        display_attribute_format(attr.attribute_info.data_format),
                                        attr.attribute_info.description,
                                    ]),
                                ));
                            }
                        }
                        Err(err) => {
                            self.stateful_table_items.push((
                                RowId::default(),
                                Row::new(vec![
                                    format!("Error retrieving info: {}", err),
                                    "".to_string(),
                                ]),
                            ));
                        }
                    }
                }
            }
            DeviceDisplay::Empty => {}
        }
        self.stateful_table.select(Some(0));
    }

    fn draw_right<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        let selected_device = match shared_view_state.selected_device.clone() {
            Some(device_name) => device_name,
            None => String::from(""),
        };
        let selected_device = match self.device_display {
            DeviceDisplay::Commands => format!(" Commands for device: {}", selected_device),
            DeviceDisplay::Attributes => format!(" Attributes for device: {}", selected_device),
            DeviceDisplay::Empty => format!(" Selected: {}", selected_device),
        };

        let header = match self.device_display {
            DeviceDisplay::Commands => vec!["Name", "Type In", "Type Out"],
            DeviceDisplay::Attributes => vec!["Name", "Type", "Format", "Description"],
            DeviceDisplay::Empty => vec![],
        };

        let widths = match self.device_display {
            DeviceDisplay::Commands => {
                let size_a = area.width / 3 + 1;
                let size_b = area.width / 3;
                let size_c = area.width / 3;
                vec![
                    Constraint::Length(size_a),
                    Constraint::Length(size_b),
                    Constraint::Length(size_c),
                ]
            }
            DeviceDisplay::Attributes => {
                let size_a = area.width / 3;
                let size_b = area.width / 6;
                let size_c = area.width / 6;
                let size_d = area.width / 3;
                vec![
                    Constraint::Length(size_a),
                    Constraint::Length(size_b),
                    Constraint::Length(size_c),
                    Constraint::Length(size_d),
                ]
            }
            DeviceDisplay::Empty => vec![],
        };

        // Column widths
        let table_items: Vec<Row> = self
            .stateful_table_items
            .iter()
            .cloned()
            .map(|entry| entry.1)
            .collect();

        let table = Table::new(table_items)
            .style(Style::default().fg(Color::White))
            .header(
                Row::new(header)
                    .style(Style::default().fg(Color::LightCyan))
                    .bottom_margin(1),
            )
            .block(Block::default().title(selected_device))
            .widths(&widths)
            .column_spacing(1)
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">>");

        f.render_stateful_widget(table, area, &mut self.stateful_table.clone());
    }

    fn handle_event_left(&mut self, key_event: &KeyEvent, shared_view_state: &mut SharedViewState) {
        match key_event.code {
            KeyCode::Left => {
                self.stateful_tree.left();
            }
            KeyCode::Right => {
                self.stateful_tree.right();
                // self.stateful_tree.open();
                if shared_view_state.selected_device.is_some() {
                    self.focus = Focus::Right;
                    self.device_display = DeviceDisplay::Attributes;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Attributes);
                    // self.stateful_table.select(Some(0));
                } else {
                    self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
                }
            }
            KeyCode::Down => {
                self.stateful_tree.next();
                shared_view_state.selected_device = None;
                self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
            }
            KeyCode::Up => {
                self.stateful_tree.previous();
                shared_view_state.selected_device = None;
                self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
            }
            KeyCode::Char('c') => {
                if shared_view_state.selected_device.is_some() && self.focus == Focus::Right {
                    self.populate_device_items(shared_view_state, DeviceDisplay::Commands);
                    self.stateful_table.select(Some(0));
                }
            }
            KeyCode::Char('a') => {
                if shared_view_state.selected_device.is_some() && self.focus == Focus::Right {
                    self.populate_device_items(shared_view_state, DeviceDisplay::Attributes);
                    self.stateful_table.select(Some(0));
                }
            }
            _ => {}
        }
    }

    fn handle_event_right(
        &mut self,
        key_event: &KeyEvent,
        shared_view_state: &mut SharedViewState,
    ) {
        match key_event.code {
            KeyCode::Up => {
                if let Some(current_selected) = self.stateful_table.selected() {
                    if current_selected > 0 {
                        self.stateful_table.select(Some(current_selected - 1));
                    }
                    if current_selected == 0 {
                        self.stateful_table
                            .select(Some(self.stateful_table_items.len() - 1));
                    }
                }
            }
            KeyCode::Down => {
                if let Some(current_selected) = self.stateful_table.selected() {
                    if current_selected == self.stateful_table_items.len() - 1 {
                        self.stateful_table.select(Some(0));
                    } else {
                        self.stateful_table.select(Some(current_selected + 1));
                    }
                }
            }

            KeyCode::Left => {
                self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
                self.focus = Focus::Left;
                self.stateful_table_items.clear();
                self.device_display = DeviceDisplay::Empty;
            }
            KeyCode::Enter => {
                if self.device_display == DeviceDisplay::Attributes {
                    if let Some(current_position) = self.stateful_table.selected() {
                        if let Some(attr_row) = self.stateful_table_items.get(current_position) {
                            shared_view_state.add_watch_attribute(attr_row.0.name.clone());
                            shared_view_state.current_view = View::WatchList;
                        }
                    }
                }
                if self.device_display == DeviceDisplay::Commands {
                    if let Some(current_position) = self.stateful_table.selected() {
                        if let Some(command_row) = self.stateful_table_items.get(current_position) {
                            shared_view_state.executed_commands.current_command =
                                Some(command_row.0.name.clone());
                            shared_view_state.executed_commands.current_command_in_type =
                                command_row.0.in_type;
                            shared_view_state.current_view = View::Command;
                        }
                    }
                }
            }
            KeyCode::Char('c') => {
                if shared_view_state.selected_device.is_some() {
                    self.device_display = DeviceDisplay::Commands;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Commands);
                }
            }
            KeyCode::Char('a') => {
                if shared_view_state.selected_device.is_some() {
                    self.device_display = DeviceDisplay::Attributes;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Attributes);
                }
            }
            _ => {}
        }
    }
}

impl From<ViewExplorerHome<'_>> for usize {
    fn from(_item: ViewExplorerHome<'_>) -> usize {
        1
    }
}

impl Draw for ViewExplorerHome<'_> {
    fn get_view_menu_items(&self, shared_view_state: &mut SharedViewState) -> Vec<MenuOption> {
        let mut items = vec![MenuOption {
            key: "←,↑,→,↓".to_string(),
            description: "Navigate tree".to_string(),
        }];

        if shared_view_state.selected_device.is_some()
            && self.focus == Focus::Right
            && self.device_display == DeviceDisplay::Commands
        {
            items.push(MenuOption {
                key: "a".to_string(),
                description: "Attribute List".to_string(),
            });
            items.push(MenuOption {
                key: "Enter".to_string(),
                description: "Execute command".to_string(),
            });
        }

        if shared_view_state.selected_device.is_some()
            && self.focus == Focus::Right
            && self.device_display == DeviceDisplay::Attributes
        {
            items.push(MenuOption {
                key: "c".to_string(),
                description: "Command List".to_string(),
            });
        }
        if shared_view_state.selected_device.is_some()
            && self.focus == Focus::Right
            && self.device_display == DeviceDisplay::Attributes
        {
            items.push(MenuOption {
                key: "ENTER".to_string(),
                description: "Watch Attribute".to_string(),
            });
        }
        items
    }

    fn handle_event(
        &mut self,
        key_event: &KeyEvent,
        shared_view_state: &mut SharedViewState,
    ) -> usize {
        if self.focus == Focus::Left {
            self.handle_event_left(key_event, shared_view_state);
        } else {
            self.handle_event_right(key_event, shared_view_state);
        }

        let selected = self.stateful_tree.state.selected();

        if selected.len() == 3 {
            let domain_ix = selected[0];
            let family_ix = selected[1];
            let member_ix = selected[2];

            if let Some(domain) = shared_view_state.tango_devices_lookup.get_by_ix(domain_ix) {
                if let Some(family) = domain.get_by_ix(family_ix) {
                    if let Some(member) = family.get_by_ix(member_ix) {
                        shared_view_state.selected_device = Some(member.device_name);
                    }
                }
            }
        } else {
            shared_view_state.selected_device = None;
        }
        0
    }

    fn draw_explorer<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        // area.width
        let length_left = area.width / 3;
        let length_right = area.width - length_left;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(length_left),
                    Constraint::Length(length_right),
                ]
                .as_ref(),
            )
            .split(area);
        self.draw_left(f, chunks[0], shared_view_state);
        self.draw_right(f, chunks[1], shared_view_state);
    }
}

impl<'a> From<usize> for ViewExplorerHome<'a> {
    fn from(_item: usize) -> Self {
        ViewExplorerHome::new(&TangoDevicesLookup::default())
    }
}
