use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Row, Table, Widget},
};

use crate::sysinfo_thread::System;

pub struct ProcessList<'a> {
    current_data: &'a System,
    usernames: HashMap<u32, String>,
}

impl<'a> ProcessList<'a> {
    pub fn new(data: &'a System) -> Self {
        ProcessList {
            current_data: data,
            usernames: HashMap::default(),
        }
    }
}

impl<'a> Widget for &mut ProcessList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut max_user = 0;

        let rows = if let Some(processes) = &self.current_data.processes {
            processes
                .iter()
                .map(|p| {
                    let style = if p.kernel_thread {
                        Style::default().gray()
                    } else {
                        Style::default().cyan()
                    };
                    let user = if let Some(user) = p.user {
                        if let Some(name) = self.usernames.get(&user) {
                            name.clone()
                        } else {
                            let name =
                                crate::utils::get_username_from_uid(user).unwrap_or_default();
                            max_user = max_user.max(name.len());
                            self.usernames.insert(user, name.clone());

                            name
                        }
                    } else {
                        String::default()
                    };
                    Row::new(vec![
                        p.pid.to_string(),
                        user,
                        p.name.clone(),
                        human_bytes::human_bytes(p.virtual_memory as f64),
                        human_bytes::human_bytes(p.memory as f64),
                        format!("{:.1}%", p.cpu_usage),
                        p.command.clone(),
                    ])
                    .style(style)
                })
                .collect()
        } else {
            Vec::default()
        };

        max_user = max_user.min(10);

        let widths = [
            Constraint::Max(6),
            Constraint::Max(max_user as u16),
            Constraint::Max(16),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Fill(1),
        ];

        Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec![
                    "PID", "User", "Name", "Virt", "Res", "CPU%", "Command",
                ])
                .style(Style::new().bold()),
            )
            .highlight_style(Style::new().reversed())
            .block(Block::bordered())
            .render(area, buf);
    }
}
