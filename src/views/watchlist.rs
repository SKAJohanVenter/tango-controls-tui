use crate::views::{Draw, SharedViewState};
use std::convert::{From, Into};
use tui::{
    backend::Backend,
    layout::Constraint,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Row, Table, TableState},
    Frame,
};

#[derive(Default, Debug)]
pub struct ViewWatchList {
    id: usize,
    stateful_table: TableState,
}

impl ViewWatchList {
    pub fn new(id: usize) -> ViewWatchList {
        ViewWatchList {
            id,
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
        let w = &shared_view_state.watch_list.lock().unwrap();
        for (device_name, attr_map) in w.iter() {
            for (attr_name, attr_value) in attr_map {
                if let Some(attr_value) = attr_value {
                    table_items.push(Row::new(vec![
                        device_name.clone(),
                        attr_name.clone(),
                        attr_value.clone(),
                    ]));
                } else {
                    table_items.push(Row::new(vec![
                        device_name.clone(),
                        attr_name.clone(),
                        "N/A".to_string(),
                    ]));
                }
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
            .block(Block::default().title(" Watched values"))
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
        ViewWatchList::new(1)
    }
}

impl Into<usize> for ViewWatchList {
    fn into(self) -> usize {
        1
    }
}

#[test]
fn test_watchlist() {
    let id: usize = 1;
    let vwl: ViewWatchList = id.into();
    assert_eq!(vwl.id, 1);

    let id: usize = 5;
    let vwl: ViewWatchList = id.into();
    assert_eq!(vwl.id, 1);

    let id: usize = vwl.into();
    assert_eq!(id, 1);
}
