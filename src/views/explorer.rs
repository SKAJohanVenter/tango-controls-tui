use crate::stateful_tree::StatefulTree;
use crate::tango_utils::TangoDevicesLookup;
use crate::tango_utils::{GetTreeItems, TreeSelection};
use crate::views::{Draw, MenuOption, SharedViewState};
use crossterm::event::{KeyCode, KeyEvent};

use ratatui::widgets::Padding;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use std::convert::From;
use tango_controls_client_sys::types::CmdArgType;
use tui_tree_widget::Tree;

use super::View;

#[derive(PartialEq)]
enum Focus {
    Left,
    Right,
}
// #[derive(PartialEq)]
// enum DeviceDisplay {
//     Commands,
//     Attributes,
//     Empty,
// }

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
}

impl<'a> ViewExplorerHome<'a> {
    pub fn new(tdl: &TangoDevicesLookup<'a>) -> ViewExplorerHome<'a> {
        ViewExplorerHome {
            stateful_tree: StatefulTree::with_items(tdl.get_tree_items()),
            // domains: tdl.domains.clone(),
            focus: Focus::Left,
            stateful_table: TableState::default(),
            stateful_table_items: Vec::new(),
            // device_display: DeviceDisplay::Empty,
        }
    }

    fn draw_left(&self, f: &mut Frame, area: Rect, _shared_view_state: &mut SharedViewState) {
        let b: Block<'_> = Block::default().borders(Borders::ALL).title("Device Tree");
        let mut items = Tree::new(self.stateful_tree.items.to_vec())
            .unwrap()
            .block(b);
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

    fn populate_device_items(&mut self, _shared_view_state: &SharedViewState) {
        self.stateful_table_items.clear();
        self.stateful_table.select(Some(0));
    }

    fn draw_right(&self, f: &mut Frame, area: Rect, shared_view_state: &mut SharedViewState) {
        let selected_ix_vec = self.stateful_tree.state.selected();
        let tree_selection: TreeSelection = shared_view_state
            .tango_devices_lookup
            .get_tree_selection(selected_ix_vec);

        let mut rows: Vec<Row> = Vec::new();
        let mut header = vec!["", ""];
        let mut title = "".to_string();

        match tree_selection {
            TreeSelection::Attribute(device_name, device_attribute) => {
                let attr_name = device_attribute.attribute_info.name.clone();
                let attr_rows: Vec<Row> = (*device_attribute).into();
                rows.extend(attr_rows);
                header = vec!["Name", "Value"];
                title = format!("  {}/{}", device_name, attr_name);
                shared_view_state.selected_device = Some(device_name);
            }
            TreeSelection::Command(device_name, command_info) => {
                let command_name = command_info.cmd_name.clone();
                let command_rows: Vec<Row> = TangoDevicesLookup::command_info_to_rows(command_info);
                rows.extend(command_rows);
                header = vec!["Name", "Value"];
                title = format!("  Commands for {}/{}", device_name, command_name);
                shared_view_state.selected_device = Some(device_name);
            }
            _ => shared_view_state.selected_device = None,
        }

        let widths = vec![
            Constraint::Length(area.width / 2),
            Constraint::Length(area.width / 2),
        ];

        let table = Table::new(rows.to_owned(), widths)
            .style(Style::default().fg(Color::White))
            .header(
                Row::new(header)
                    .style(Style::default().fg(Color::LightCyan))
                    .bottom_margin(1),
            )
            .block(
                Block::default()
                    .title(title)
                    .padding(Padding::new(2, 2, 1, 1)),
            )
            // .widths(&widths)
            .column_spacing(1)
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(table, area, &mut TableState::default());
    }

    fn handle_event_left(&mut self, key_event: &KeyEvent, shared_view_state: &mut SharedViewState) {
        match key_event.code {
            KeyCode::Left => {
                self.stateful_tree.left();
            }
            KeyCode::Right => {
                self.stateful_tree.right();
            }
            KeyCode::Down => {
                self.stateful_tree.next();
                shared_view_state.selected_device = None;
                self.populate_device_items(shared_view_state);
            }
            KeyCode::Up => {
                self.stateful_tree.previous();
                shared_view_state.selected_device = None;
                self.populate_device_items(shared_view_state);
            }

            KeyCode::Enter | KeyCode::Char('w') => {
                let selected_ix_vec = self.stateful_tree.state.selected();
                let tree_selection: TreeSelection = shared_view_state
                    .tango_devices_lookup
                    .get_tree_selection(selected_ix_vec);

                match tree_selection {
                    TreeSelection::Attribute(_, device_attribute) => {
                        shared_view_state.add_watch_attribute(device_attribute.attribute_info.name);
                        shared_view_state.current_view = View::WatchList;
                    }
                    _ => shared_view_state.current_view = View::Explorer,
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
    fn get_view_menu_items(&self, _shared_view_state: &mut SharedViewState) -> Vec<MenuOption> {
        let items = vec![
            MenuOption {
                key: "←,↑,→,↓".to_string(),
                description: "Navigate tree".to_string(),
            },
            MenuOption {
                key: "w".to_string(),
                description: "Watch attribute".to_string(),
            },
        ];
        items
    }

    fn handle_event(
        &mut self,
        key_event: &KeyEvent,
        shared_view_state: &mut SharedViewState,
    ) -> usize {
        self.handle_event_left(key_event, shared_view_state);
        0
    }

    fn draw_explorer(&self, f: &mut Frame, area: Rect, shared_view_state: &mut SharedViewState) {
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
