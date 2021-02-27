use std::ops::Index;

use crate::{app::App, tango_utils::GetTreeItems};
use log::{info, warn};
// use tui-tree-widget::
use crate::stateful_tree::StatefulTree;
use crate::tango_utils::TangoDevicesLookup;
use crossterm::event::{KeyCode, KeyEvent};
use tui::symbols::line::DOUBLE_VERTICAL;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};
use tui_tree_widget::{Tree, TreeState};

// #[derive(Debug)]
pub enum View<'a> {
    ExplorerHome(ViewExplorerHome<'a>),
    ExplorerCommands(ViewExplorerCommands),
    ExplorerAttributes(ViewExplorerAttributes),
    WatchList(ViewWatchList),
}

// #[derive(Debug, Default)]
pub struct ViewExplorerHome<'a> {
    id: usize,
    pub stateful_tree: StatefulTree<'a>,
    pub selected_tango_device: Option<String>,
}

impl<'a> ViewExplorerHome<'a> {
    pub fn new(id: usize, tdl: &TangoDevicesLookup<'a>) -> ViewExplorerHome<'a> {
        ViewExplorerHome {
            id,
            // stateful_tree, // stateful_tree: StatefulTree::with_items(vec![]), // stateful_tree: StatefulTree::with_items(app.tango_devices_lookup.get_tree_items()),
            stateful_tree: StatefulTree::with_items(tdl.get_tree_items()),
            selected_tango_device: None,
        }
    }

    pub fn get_stateful_tree(&self) -> &StatefulTree<'a> {
        &self.stateful_tree
    }
}

#[derive(Default, Debug)]
pub struct ViewExplorerCommands {
    id: usize,
}

impl ViewExplorerCommands {
    pub fn new(id: usize) -> ViewExplorerCommands {
        ViewExplorerCommands { id }
    }
}

#[derive(Default, Debug)]
pub struct ViewExplorerAttributes {
    id: usize,
}

impl ViewExplorerAttributes {
    pub fn new(id: usize) -> ViewExplorerAttributes {
        ViewExplorerAttributes { id }
    }
}

#[derive(Default, Debug)]
pub struct ViewWatchList {
    id: usize,
}

impl ViewWatchList {
    pub fn new(id: usize) -> ViewWatchList {
        ViewWatchList { id }
    }
}

pub trait Draw {
    fn set_selected_device(&mut self, state: &TreeState) {}

    fn draw_header<B: Backend>(&self, f: &mut Frame<B>, area: Rect, app: &App) {
        let tango_host_text = Paragraph::new(format!("TANGO_HOST: {}", app.tango_host))
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Left)
            .block(Block::default().style(Style::default().fg(Color::White)));
        f.render_widget(tango_host_text, area);
    }

    fn draw_menu<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let table = Table::new(vec![
            Row::new(vec![Cell::from("")]),
            Row::new(vec![
                Cell::from(Spans::from(vec![
                    Span::styled("<", Style::default().fg(Color::LightCyan)),
                    Span::styled("q", Style::default().fg(Color::White)),
                    Span::styled(">", Style::default().fg(Color::LightCyan)),
                ])),
                Cell::from("Quit").style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from(Spans::from(vec![
                    Span::styled("<", Style::default().fg(Color::LightCyan)),
                    Span::styled("TAB", Style::default().fg(Color::White)),
                    Span::styled(">", Style::default().fg(Color::LightCyan)),
                ])),
                Cell::from("Toggle tabs").style(Style::default().fg(Color::White)),
            ])
            .bottom_margin(1),
            // If a Row need to display some content over multiple lines, you just have to change
            // its height.
        ])
        // You can set the style of the entire Table.
        .style(Style::default().fg(Color::White))
        // It has an optional header, which is simply a Row always visible at the top.
        // As any other widget, a Table can be wrapped in a Block.
        // Columns widths are constrained in the same way as Layout...
        .widths(&[
            Constraint::Length(5),
            Constraint::Length(15),
            Constraint::Length(15),
        ])
        // ...and they can be separated by a fixed spacing.
        .column_spacing(1);
        f.render_widget(table, area);
    }

    fn draw_tabs<B: Backend>(&self, f: &mut Frame<B>, area: Rect, current_view_ix: usize) {
        let selected_tab = match current_view_ix {
            3 => 1,
            _ => 0,
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

    fn draw_explorer<B: Backend>(&self, _f: &mut Frame<B>, _area: Rect, _app: &App) {}

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
        tango_devices_lookup: &TangoDevicesLookup,
    ) -> usize {
        0
    }

    fn draw_body<B: Backend>(&self, f: &mut Frame<B>, area: Rect, app: &App) {
        self.draw_explorer(f, area, app);
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>, app: &App) {
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

        self.draw_header(f, chunks[0], app);
        self.draw_menu(f, chunks[1]);
        self.draw_tabs(f, chunks[2], app.current_view_ix);
        self.draw_body(f, chunks[3], app);
        self.draw_footer(f, chunks[4]);
    }
}

impl Draw for ViewExplorerHome<'_> {
    fn set_selected_device(&mut self, state: &TreeState) {
        let selected = state.selected();
        if selected.len() == 3 {
            self.selected_tango_device = Some(String::from("AA"));
        } else {
            self.selected_tango_device = None;
        }
    }

    fn handle_event(
        &mut self,
        key_event: &KeyEvent,
        tango_devices_lookup: &TangoDevicesLookup,
    ) -> usize {
        match key_event.code {
            KeyCode::Left => {
                self.stateful_tree.close();
            }
            KeyCode::Right => {
                self.stateful_tree.open();
            }
            KeyCode::Down => {
                self.stateful_tree.next();
            }
            KeyCode::Up => {
                self.stateful_tree.previous();
            }
            _ => {}
        }
        let selected = self.stateful_tree.state.selected();

        if selected.len() == 3 {
            let domain_ix = selected[0];
            let family_ix = selected[1];
            let member_ix = selected[2];

            if let Some(domain) = tango_devices_lookup.get_by_ix(domain_ix) {
                if let Some(family) = domain.get_by_ix(family_ix) {
                    if let Some(member) = family.get_by_ix(member_ix) {
                        self.selected_tango_device = Some(member.device_name.clone());
                    }
                }
            }
        } else {
            self.selected_tango_device = None;
        }
        println!("{:#?}", self.selected_tango_device);
        0
    }

    fn draw_explorer<B: Backend>(&self, f: &mut Frame<B>, area: Rect, app: &App) {
        let items = Tree::new(self.stateful_tree.items.to_vec())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{:?}", self.stateful_tree.state)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

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

        let left = Paragraph::new("Left")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );

        let right = Paragraph::new("Right")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            );
        // f.render_widget(left, chunks[0]);
        // f.render_stateful_widget(items, chunks[0], &mut self.stateful_tree.state);
        f.render_stateful_widget(items, chunks[0], &mut self.stateful_tree.state.clone());
        f.render_widget(right, chunks[1]);
    }
}

impl Draw for ViewExplorerCommands {}
impl Draw for ViewExplorerAttributes {}

impl Draw for ViewWatchList {
    fn draw_body<B: Backend>(&self, f: &mut Frame<B>, area: Rect, app: &App) {
        self.draw_watchlist(f, area);
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
