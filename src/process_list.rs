use std::{cmp::Ordering, collections::HashMap};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Row, StatefulWidget, Table, TableState, Widget},
};

use crate::{
    proc::{System, process_info::ProcessType},
    utils::human_bytes,
};

#[derive(Debug, Default)]
pub struct ProcessList {
    current_data: System,
    usernames: HashMap<u32, String>,
    show_kernel_threads: bool,
    state: TableState,
}

impl ProcessList {
    pub fn new(show_kernel_threads: bool) -> Self {
        ProcessList {
            current_data: System::default(),
            usernames: HashMap::default(),
            show_kernel_threads,
            state: TableState::default(),
        }
    }

    pub fn toggle_kernel_threads(&mut self) {
        self.show_kernel_threads = !self.show_kernel_threads;
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => self.state.select_next(),
            _ => {}
        }
    }

    pub fn set_data(&mut self, data: System) {
        self.current_data = data;
        self.current_data.processes.sort_by(|a, b| {
            if let Some(cmp) = a.cpu_usage.partial_cmp(&b.cpu_usage) {
                cmp.reverse()
            } else {
                Ordering::Equal
            }
        });
    }

    pub fn data(&self) -> &System {
        &self.current_data
    }
}

impl Widget for &mut ProcessList {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut max_user = 0;

        let rows: Vec<Row> = self
            .current_data
            .processes
            .iter()
            .filter_map(|p| {
                let style = match p.process_type {
                    ProcessType::Task => Style::default().cyan(),
                    ProcessType::KernelThread => {
                        if !self.show_kernel_threads {
                            return None;
                        }
                        Style::default().gray()
                    }
                    ProcessType::Thread => Style::default(),
                };

                let style = if let crate::proc::state::State::Running = p.state {
                    style.bold()
                } else {
                    style
                };

                let user = if let Some(user) = p.uid {
                    if let Some(name) = self.usernames.get(&user) {
                        name.clone()
                    } else {
                        let name = crate::utils::get_username_from_uid(user).unwrap_or_default();
                        max_user = max_user.max(name.len());
                        self.usernames.insert(user, name.clone());

                        name
                    }
                } else {
                    String::default()
                };
                Some(
                    Row::new(vec![
                        format!("{:>7}", p.pid),
                        user,
                        p.name.clone(),
                        human_bytes(p.virtual_memory, true),
                        human_bytes(p.memory, true),
                        p.state.to_string(),
                        format!("{:>5.1}%", p.cpu_usage.unwrap_or_default()),
                        p.cmdline.clone(),
                    ])
                    .style(style),
                )
            })
            .collect();

        max_user = max_user.min(10);

        let widths = [
            Constraint::Max(7),
            Constraint::Max(max_user as u16),
            Constraint::Max(15),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Fill(1),
        ];

        let table = Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec![
                    "PID", "User", "Name", "Virt", "Res", "S", "CPU%", "Command",
                ])
                .style(Style::new().underlined()),
            )
            .row_highlight_style(Style::new().reversed());

        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}
