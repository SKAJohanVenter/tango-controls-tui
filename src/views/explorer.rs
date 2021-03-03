use crate::tango_utils::{get_attribute_list, get_command_list, GetTreeItems};
use crate::views::{Draw, MenuOption, SharedViewState};
// use tui-tree-widget::
use crate::stateful_tree::StatefulTree;
use crate::tango_utils::TangoDevicesLookup;
use crossterm::event::{KeyCode, KeyEvent};
use std::convert::{From, Into};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};
use tui_tree_widget::Tree;

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

pub struct ViewExplorerHome<'a> {
    id: usize,
    stateful_tree: StatefulTree<'a>,
    focus: Focus,
    stateful_table: TableState,
    stateful_table_items: Vec<(String, Row<'a>)>,
    device_display: DeviceDisplay,
}

impl<'a> ViewExplorerHome<'a> {
    pub fn new(id: usize, tdl: &TangoDevicesLookup<'a>) -> ViewExplorerHome<'a> {
        ViewExplorerHome {
            id,
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
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
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
        if device_display == DeviceDisplay::Attributes {
            if let Some(current_device) = shared_view_state.current_selected_device.clone() {
                match get_attribute_list(current_device.as_str()) {
                    Ok(attributes) => {
                        for attr in attributes {
                            self.stateful_table_items.push((
                                format!("{}", attr.name),
                                Row::new(vec![attr.name, attr.description]),
                            ));
                        }
                    }
                    Err(err) => {
                        self.stateful_table_items.push((
                            "".to_string(),
                            Row::new(vec![
                                format!("Error retrieving info: {}", err),
                                "".to_string(),
                            ]),
                        ));
                    }
                }
            }
        }
        if device_display == DeviceDisplay::Commands {
            if let Some(current_device) = shared_view_state.current_selected_device.clone() {
                match get_command_list(current_device.as_str()) {
                    Ok(commands) => {
                        for comm in commands {
                            self.stateful_table_items.push((
                                format!("{}", comm.name),
                                Row::new(vec![
                                    comm.name,
                                    format!("{:?}", comm.in_type),
                                    format!("{:?}", comm.out_type),
                                ]),
                            ));
                        }
                    }
                    Err(err) => {
                        self.stateful_table_items.push((
                            "".to_string(),
                            Row::new(vec![
                                format!("Error retrieving info: {}", err),
                                "".to_string(),
                            ]),
                        ));
                    }
                }
            }
        }
        self.stateful_table.select(Some(0));
    }

    fn draw_right<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        let mut selected_device = match shared_view_state.current_selected_device.clone() {
            Some(device_name) => device_name,
            None => String::from(""),
        };
        selected_device = format!(" Selected: {}", selected_device);

        let header = match self.device_display {
            DeviceDisplay::Commands => {
                vec!["Name", "Type In", "Type Out"]
            }
            DeviceDisplay::Attributes => {
                vec!["Name", "Description"]
            }
            DeviceDisplay::Empty => {
                vec![]
            }
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
                let size_a = area.width / 2 + 1;
                let size_b = area.width / 2;
                vec![Constraint::Length(size_a), Constraint::Length(size_b)]
            }
            DeviceDisplay::Empty => {
                vec![]
            }
        };

        // Column widths
        let table_items: Vec<Row> = self
            .stateful_table_items
            .iter()
            .cloned()
            .map(|entry| entry.1)
            .collect();

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
            .block(Block::default().title(selected_device))
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

    fn handle_event_left(&mut self, key_event: &KeyEvent, shared_view_state: &mut SharedViewState) {
        match key_event.code {
            KeyCode::Left => {
                self.stateful_tree.close();
            }
            KeyCode::Right => {
                self.stateful_tree.open();
                if shared_view_state.current_selected_device.is_some() {
                    self.focus = Focus::Right;
                    self.device_display = DeviceDisplay::Attributes;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Attributes);
                    self.stateful_table.select(Some(0));
                } else {
                    self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
                }
            }
            KeyCode::Down => {
                self.stateful_tree.next();
                shared_view_state.current_selected_device = None;
                self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
            }
            KeyCode::Up => {
                self.stateful_tree.previous();
                shared_view_state.current_selected_device = None;
                self.populate_device_items(shared_view_state, DeviceDisplay::Empty);
            }
            KeyCode::Char('c') => {
                if shared_view_state.current_selected_device.is_some() {
                    self.focus = Focus::Right;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Commands);
                    self.stateful_table.select(Some(0));
                }
            }
            KeyCode::Char('a') => {
                if shared_view_state.current_selected_device.is_some() {
                    self.focus = Focus::Right;
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
                }
            }
            KeyCode::Down => {
                if let Some(current_selected) = self.stateful_table.selected() {
                    if current_selected == self.stateful_table_items.len() {
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
            KeyCode::Char('w') => {
                if self.device_display == DeviceDisplay::Attributes {
                    if let Some(current_position) = self.stateful_table.selected() {
                        if let Some(attr_row) = self.stateful_table_items.get(current_position) {
                            shared_view_state.add_watch_attribute(attr_row.0.clone());
                        }
                    }
                }
            }
            KeyCode::Char('c') => {
                if shared_view_state.current_selected_device.is_some() {
                    self.device_display = DeviceDisplay::Commands;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Commands);
                }
            }
            KeyCode::Char('a') => {
                if shared_view_state.current_selected_device.is_some() {
                    self.device_display = DeviceDisplay::Attributes;
                    self.populate_device_items(shared_view_state, DeviceDisplay::Attributes);
                }
            }
            _ => {}
        }
    }
}

impl Draw for ViewExplorerHome<'_> {
    fn get_view_menu_items(&self, shared_view_state: &mut SharedViewState) -> Vec<MenuOption> {
        let mut items = vec![MenuOption {
            key: "←,↑,→,↓".to_string(),
            description: "Navigate tree".to_string(),
        }];

        if shared_view_state.current_selected_device.is_some() {
            items.push(MenuOption {
                key: "a".to_string(),
                description: "Attribute List".to_string(),
            });

            items.push(MenuOption {
                key: "c".to_string(),
                description: "Command List".to_string(),
            });
        }
        items
    }

    fn handle_event(
        &mut self,
        key_event: &KeyEvent,
        tango_devices_lookup: &TangoDevicesLookup,
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

            if let Some(domain) = tango_devices_lookup.get_by_ix(domain_ix) {
                if let Some(family) = domain.get_by_ix(family_ix) {
                    if let Some(member) = family.get_by_ix(member_ix) {
                        shared_view_state.current_selected_device =
                            Some(member.device_name.clone());
                    }
                }
            }
        } else {
            shared_view_state.current_selected_device = None;
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
            .margin(0)
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
        ViewExplorerHome::new(0, &TangoDevicesLookup::default())
    }
}

impl<'a> Into<usize> for ViewExplorerHome<'a> {
    fn into(self) -> usize {
        0
    }
}

#[test]
fn test_exporer<'a>() {
    let id: usize = 0;
    let ve: ViewExplorerHome = id.into();
    assert_eq!(ve.id, 0);

    let id: usize = 5;
    let ve: ViewExplorerHome = id.into();
    assert_eq!(ve.id, 0);

    let id: usize = ve.into();
    assert_eq!(id, 0);
}
