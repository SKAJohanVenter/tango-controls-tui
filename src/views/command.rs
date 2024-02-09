use crate::Event;
use std::{collections::BTreeMap, sync::mpsc};
use tango_controls_client_sys::types::CmdArgType;

// use super::{MenuOption, View};

// #[derive(Debug, Default)]
// enum Focus {
//     #[default]
//     Input,
// }

#[derive(Debug)]
pub struct ExecutedCommand {
    pub command: String,
    pub parameter: String,
    pub result: String,
    pub device_name: String,
}

#[derive(Debug)]
pub struct ExecutedCommands {
    pub executed_commands: BTreeMap<u128, ExecutedCommand>,
    pub current_command: Option<String>,
    pub current_command_in_type: Option<CmdArgType>,
    pub current_parsed_parameter: Option<String>,
    pub current_parsed_error: Option<String>,
    pub current_parameter: Option<String>,
    pub tx_commands: mpsc::Sender<Event>,
    pub current_device: Option<String>,
}

// impl ExecutedCommands {
//     pub fn get_millis_since_epoch(&self) -> u128 {
//         let start = SystemTime::now();
//         start
//             .duration_since(UNIX_EPOCH)
//             .expect("Time went backwards")
//             .as_millis()
//     }

//     pub fn new(tx_commands: mpsc::Sender<Event>) -> Self {
//         Self {
//             executed_commands: BTreeMap::default(),
//             current_command: None,
//             current_parameter: None,
//             current_device: None,
//             current_command_in_type: None,
//             current_parsed_error: None,
//             current_parsed_parameter: None,
//             tx_commands,
//         }
//     }

//     pub fn execute_command(&mut self, device_name: String, command: String, parameter: String) {
//         let seconds_since_epoch = self.get_millis_since_epoch();
//         let execute_command = ExecutedCommand {
//             command: command.clone(),
//             parameter: parameter.clone(),
//             device_name: device_name.clone(),
//             result: String::from("In Progress"),
//         };
//         self.executed_commands
//             .insert(seconds_since_epoch, execute_command);

//         let tx_commands = self.tx_commands.clone();
//         thread::spawn(move || {
//             let res = match tango_utils::execute_tango_command(
//                 device_name.as_str(),
//                 command.as_str(),
//                 parameter.as_str(),
//             ) {
//                 Ok(command_data) => format!("{:?}", command_data),
//                 Err(err) => {
//                     error!("Command Error {}", err);
//                     err.to_string()
//                 }
//             };
//             match tx_commands.send(Event::UpdateCommandResult(seconds_since_epoch, res)) {
//                 Ok(_) => {}
//                 Err(err) => {
//                     error!("Could not send result {}", err)
//                 }
//             }
//         });
//     }
// }

// #[derive(Default, Debug)]
// pub struct ViewCommand {
//     focus: Focus,
//     input: String,
// }

// impl ViewCommand {
//     pub fn new() -> ViewCommand {
//         ViewCommand {
//             focus: Focus::Input,
//             input: String::from("< Enter parameter >"),
//         }
//     }

//     fn handle_event(&mut self, key_event: &KeyEvent, shared_view_state: &mut SharedViewState) {
//         let paramater_string = String::from("< Enter parameter >");

//         if shared_view_state.selected_device.is_some()
//             && shared_view_state
//                 .executed_commands
//                 .current_command
//                 .is_some()
//         {
//             match key_event.code {
//                 KeyCode::Left | KeyCode::Right => {
//                     if self.input == paramater_string {
//                         self.input.clear();
//                     }
//                 }
//                 KeyCode::Enter => match self.focus {
//                     Focus::Input => {
//                         if self.input == paramater_string {
//                             self.input.clear();
//                         }
//                         shared_view_state.current_view = View::ConfirmCommand;
//                         shared_view_state.executed_commands.current_parameter =
//                             Some(self.input.clone());
//                         shared_view_state.executed_commands.current_device =
//                             shared_view_state.selected_device.clone();
//                     }
//                 },
//                 KeyCode::Char(c) => match self.focus {
//                     Focus::Input => {
//                         if self.input == paramater_string {
//                             self.input.clear();
//                         }
//                         self.input.push(c);
//                     }
//                 },
//                 KeyCode::Backspace => match self.focus {
//                     Focus::Input => {
//                         if self.input == paramater_string {
//                             self.input.clear();
//                         }
//                         self.input.pop();
//                     }
//                 },
//                 _ => {}
//             }
//         }
//     }

//     fn draw_table<B: Backend>(
//         &self,
//         f: &mut Frame,
//         area: Rect,
//         shared_view_state: &mut SharedViewState,
//     ) {
//         let chunks = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
//             .split(area);

//         let title = match &shared_view_state.selected_device {
//             Some(dev) => match shared_view_state.executed_commands.current_command.clone() {
//                 Some(comm) => format!(" Device: {} - Command: {} ", dev, comm),
//                 None => format!("Device: {} - Command: <Not Selected>", dev),
//             },
//             None => "Device and Command not selected".to_string(),
//         };

//         let input = if shared_view_state.selected_device.is_none()
//             || shared_view_state
//                 .executed_commands
//                 .current_command
//                 .is_none()
//         {
//             // No device or command
//             let input = Paragraph::new("Please select a device and command from the explorer menu")
//                 .block(Block::default().borders(Borders::ALL).title(title));
//             input
//         } else {
//             let input = Paragraph::new(self.input.as_str())
//                 .block(Block::default().borders(Borders::ALL).title(title));
//             match self.focus {
//                 Focus::Input => {
//                     f.set_cursor(chunks[0].x + self.input.len() as u16 + 1, chunks[0].y + 1);
//                 }
//             };
//             input
//         };

//         let mut rows: Vec<Row> = Vec::new();
//         for (_, executed_command) in shared_view_state
//             .executed_commands
//             .executed_commands
//             .iter()
//             .rev()
//         {
//             rows.push(Row::new(vec![
//                 Cell::from(executed_command.device_name.clone()),
//                 Cell::from(executed_command.command.clone()),
//                 Cell::from(executed_command.result.clone()),
//             ]))
//         }

//         let size_a = area.width / 3;
//         let size_b = area.width / 3;
//         let size_c = area.width / 3 + 1;
//         let widths = vec![
//             Constraint::Length(size_a),
//             Constraint::Length(size_b),
//             Constraint::Length(size_c),
//         ];

//         let table = Table::new(rows)
//             .style(Style::default().fg(Color::White))
//             .block(
//                 Block::default()
//                     .borders(Borders::ALL)
//                     .style(Style::default().fg(Color::White))
//                     .border_type(BorderType::Plain)
//                     .title(" Commands"),
//             )
//             .header(Row::new(vec!["Device", "Command", "Result"]).bottom_margin(1))
//             .widths(&widths)
//             .column_spacing(1);

//         f.render_widget(input, chunks[0]);
//         f.render_widget(table, chunks[1]);
//     }
// }

// impl Draw for ViewCommand {
//     fn get_view_menu_items(&self, _shared_view_state: &mut SharedViewState) -> Vec<MenuOption> {
//         let items = vec![MenuOption {
//             key: "Enter".to_string(),
//             description: "Execute command with paramater".to_string(),
//         }];
//         items
//     }

//     fn draw_body<B: Backend>(
//         &self,
//         f: &mut Frame,
//         area: Rect,
//         shared_view_state: &mut SharedViewState,
//     ) {
//         self.draw_table(f, area, shared_view_state);
//     }

//     fn handle_event(
//         &mut self,
//         key_event: &KeyEvent,
//         shared_view_state: &mut SharedViewState,
//     ) -> usize {
//         self.handle_event(key_event, shared_view_state);
//         2
//     }
// }

// impl From<usize> for ViewCommand {
//     fn from(_item: usize) -> Self {
//         ViewCommand::new()
//     }
// }

// impl From<ViewCommand> for usize {
//     fn from(_item: ViewCommand) -> usize {
//         2
//     }
// }
