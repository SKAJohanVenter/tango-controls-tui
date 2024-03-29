use crate::{
    tango_utils::parse_command_data,
    views::{Draw, SharedViewState},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::Alignment,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::convert::From;

use super::View;

#[derive(Default, Debug)]
pub struct ViewConfirmCommand {}

impl ViewConfirmCommand {
    pub fn new() -> ViewConfirmCommand {
        ViewConfirmCommand {}
    }

    pub fn update_param_values(&self, shared_view_state: &mut SharedViewState) {
        shared_view_state.executed_commands.current_parsed_error = None;
        shared_view_state.executed_commands.current_parsed_parameter = None;

        if let Some(current_param) = shared_view_state
            .executed_commands
            .current_parameter
            .clone()
        {
            if let Some(in_type) = shared_view_state.executed_commands.current_command_in_type {
                match parse_command_data(current_param.as_str(), in_type) {
                    Ok(command_data) => {
                        shared_view_state.executed_commands.current_parsed_parameter =
                            Some(format!("{:?}", command_data))
                    }
                    Err(err) => {
                        shared_view_state.executed_commands.current_parsed_error =
                            Some(err.to_string())
                    }
                }
            }
        }
    }

    fn schedule_command(&mut self, shared_view_state: &mut SharedViewState) {
        if let Some(device) = shared_view_state.executed_commands.current_device.clone() {
            if let Some(command) = shared_view_state.executed_commands.current_command.clone() {
                if let Some(parameter) = shared_view_state
                    .executed_commands
                    .current_parameter
                    .clone()
                {
                    shared_view_state
                        .executed_commands
                        .execute_command(device, command, parameter);
                }
            }
        }
    }

    fn handle_event(&mut self, key_event: &KeyEvent, shared_view_state: &mut SharedViewState) {
        match key_event.code {
            KeyCode::Enter => {
                if shared_view_state
                    .executed_commands
                    .current_parsed_error
                    .is_none()
                {
                    self.schedule_command(shared_view_state);
                }
                shared_view_state.current_view = View::Command;
            }
            KeyCode::Char(ch) => match ch {
                'Y' | 'y' => {
                    if shared_view_state
                        .executed_commands
                        .current_parsed_error
                        .is_none()
                    {
                        self.schedule_command(shared_view_state);
                    }
                    shared_view_state.current_view = View::Command;
                }
                _ => shared_view_state.current_view = View::Command,
            },
            _ => {}
        }
    }

    fn draw_confirm<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
        parsed_command: String,
    ) {
        let create_block = |title| {
            Block::default().borders(Borders::ALL).title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
        };

        let text = vec![
            Line::from(Span::raw("")),
            Line::from(format!(
                "Execute command: {}",
                shared_view_state
                    .executed_commands
                    .current_command
                    .clone()
                    .unwrap_or_default()
            )),
            Line::from(""),
            Line::from(format!("With paramater : {}", parsed_command)),
        ];

        let paragraph = Paragraph::new(text)
            .block(create_block(" Confirm (Y)es / (N)o "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }

    fn draw_error<B: Backend>(&self, f: &mut Frame<B>, area: Rect, err: String) {
        let create_block = |title| {
            Block::default().borders(Borders::ALL).title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
        };

        let text = vec![Line::from(format!("Parameter Error: {}", err))];

        let paragraph = Paragraph::new(text)
            .block(create_block(" Error "))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

impl Draw for ViewConfirmCommand {
    fn draw_body<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        shared_view_state: &mut SharedViewState,
    ) {
        self.update_param_values(shared_view_state);
        if let Some(err) = shared_view_state
            .executed_commands
            .current_parsed_error
            .clone()
        {
            self.draw_error(f, area, err);
            return;
        }

        if let Some(param) = shared_view_state
            .executed_commands
            .current_parsed_parameter
            .clone()
        {
            self.draw_confirm(f, area, shared_view_state, param);
        }
    }

    fn handle_event(
        &mut self,
        key_event: &KeyEvent,
        shared_view_state: &mut SharedViewState,
    ) -> usize {
        self.handle_event(key_event, shared_view_state);
        2
    }
}

impl From<ViewConfirmCommand> for usize {
    fn from(_item: ViewConfirmCommand) -> usize {
        3
    }
}
