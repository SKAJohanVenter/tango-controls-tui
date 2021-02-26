// use crate::app::App;
// use tui::symbols::line::DOUBLE_VERTICAL;
// use tui::{
//     backend::Backend,
//     layout::{Alignment, Constraint, Direction, Layout, Rect},
//     style::{Color, Modifier, Style},
//     symbols,
//     text::{Span, Spans},
//     widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
//     widgets::{
//         Axis, BarChart, Block, BorderType, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List,
//         ListItem, Paragraph, Row, Sparkline, Table, Tabs, Wrap,
//     },
//     Frame,
// };

// pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
//     // let chunks = Layout::default()
//     //     .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
//     //     .split(f.size());

//     let size = f.size();

//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .margin(0)
//         .constraints(
//             [
//                 Constraint::Length(1), // TANGO HOST
//                 Constraint::Length(5), // Instructions
//                 Constraint::Length(3), // Tabs
//                 Constraint::Min(2),    // Explorer
//                 Constraint::Length(3), // Messages
//             ]
//             .as_ref(),
//         )
//         .split(size);

//     draw_header(f, chunks[0]);
//     draw_menu(f, chunks[1]);
//     draw_tabs(f, chunks[2]);
//     draw_explorer(f, chunks[3]);
//     draw_footer(f, chunks[4])

//     // f.render_widget(tango_host_text, chunks[0]);
//     // f.render_widget(table, chunks[1]);
//     // f.render_widget(footer, chunks[4]);
//     // f.render_widget(tabs, chunks[2]);
//     // f.render_widget(widget, area)

//     // let titles = app
//     //     .tabs
//     //     .titles
//     //     .iter()
//     //     .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::Green))))
//     //     .collect();
//     // let tabs = Tabs::new(titles)
//     //     .block(Block::default().borders(Borders::ALL).title(app.title))
//     //     .highlight_style(Style::default().fg(Color::Yellow))
//     //     .select(app.tabs.index);
//     // f.render_widget(tabs, chunks[0]);
//     // match app.tabs.index {
//     //     0 => draw_first_tab(f, app, chunks[1]),
//     //     1 => draw_second_tab(f, app, chunks[1]),
//     //     2 => draw_third_tab(f, app, chunks[1]),
//     //     _ => {}
//     // };
// }

// fn draw_header<B: Backend>(f: &mut Frame<B>, area: Rect) {
//     let tango_host_text = Paragraph::new("TANGO_HOST: abcd:10000")
//         .style(Style::default().fg(Color::LightCyan))
//         .alignment(Alignment::Left)
//         .block(Block::default().style(Style::default().fg(Color::White)));
//     f.render_widget(tango_host_text, area);
// }

// fn draw_menu<B: Backend>(f: &mut Frame<B>, area: Rect) {
//     let table = Table::new(vec![
//         Row::new(vec![Cell::from("")]),
//         Row::new(vec![
//             Cell::from(Spans::from(vec![
//                 Span::styled("<", Style::default().fg(Color::LightCyan)),
//                 Span::styled("TAB", Style::default().fg(Color::White)),
//                 Span::styled(">", Style::default().fg(Color::LightCyan)),
//             ])),
//             Cell::from("Toggle tabs").style(Style::default().fg(Color::White)),
//         ]),
//         Row::new(vec![
//             Cell::from(Spans::from(vec![
//                 Span::styled("<", Style::default().fg(Color::LightCyan)),
//                 Span::styled("e", Style::default().fg(Color::White)),
//                 Span::styled(">", Style::default().fg(Color::LightCyan)),
//             ])),
//             Cell::from("Explorer").style(Style::default().fg(Color::White)),
//         ]),
//         Row::new(vec![
//             Cell::from(Spans::from(vec![
//                 Span::styled("<", Style::default().fg(Color::LightCyan)),
//                 Span::styled("w", Style::default().fg(Color::White)),
//                 Span::styled(">", Style::default().fg(Color::LightCyan)),
//             ])),
//             Cell::from("Watch List").style(Style::default().fg(Color::White)),
//         ])
//         .bottom_margin(1),
//         // If a Row need to display some content over multiple lines, you just have to change
//         // its height.
//     ])
//     // You can set the style of the entire Table.
//     .style(Style::default().fg(Color::White))
//     // It has an optional header, which is simply a Row always visible at the top.
//     // As any other widget, a Table can be wrapped in a Block.
//     // Columns widths are constrained in the same way as Layout...
//     .widths(&[
//         Constraint::Length(5),
//         Constraint::Length(10),
//         Constraint::Length(10),
//     ])
//     // ...and they can be separated by a fixed spacing.
//     .column_spacing(1);
//     f.render_widget(table, area);
// }

// fn draw_tabs<B: Backend>(f: &mut Frame<B>, area: Rect) {
//     let tab_titles = ["Explorer", "Watchlist"]
//         .iter()
//         .cloned()
//         .map(Spans::from)
//         .collect();
//     let tabs = Tabs::new(tab_titles)
//         .select(1)
//         .block(Block::default().borders(Borders::ALL))
//         .style(Style::default().fg(Color::White))
//         .highlight_style(Style::default().fg(Color::Yellow))
//         .divider(DOUBLE_VERTICAL);
//     f.render_widget(tabs, area);
// }

// fn draw_explorer<B: Backend>(f: &mut Frame<B>, area: Rect) {
//     // area.width
//     let length_left = area.width / 3;
//     let length_right = area.width - length_left;
//     let chunks = Layout::default()
//         .direction(Direction::Horizontal)
//         .margin(0)
//         .constraints(
//             [
//                 Constraint::Length(length_left),
//                 Constraint::Length(length_right),
//             ]
//             .as_ref(),
//         )
//         .split(area);

//     let left = Paragraph::new("Left")
//         .style(Style::default().fg(Color::LightCyan))
//         .alignment(Alignment::Left)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .style(Style::default().fg(Color::White))
//                 .border_type(BorderType::Plain),
//         );

//     let right = Paragraph::new("Right")
//         .style(Style::default().fg(Color::LightCyan))
//         .alignment(Alignment::Left)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .style(Style::default().fg(Color::White))
//                 .border_type(BorderType::Plain),
//         );
//     f.render_widget(left, chunks[0]);
//     f.render_widget(right, chunks[1]);
// }

// fn draw_watchlist<B: Backend>(f: &mut Frame<B>, area: Rect) {}

// fn draw_footer<B: Backend>(f: &mut Frame<B>, area: Rect) {
//     let footer = Paragraph::new("Tango Controls Explorer TUI")
//         .style(Style::default().fg(Color::LightCyan))
//         .alignment(Alignment::Center)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .style(Style::default().fg(Color::White))
//                 .border_type(BorderType::Plain),
//         );
//     f.render_widget(footer, area);
// }
