use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Row, Table, Widget},
};

use crate::{sysinfo_thread::System, utils::human_bytes};

pub struct ProcessList<'a> {
    current_data: &'a System,
    usernames: HashMap<u32, String>,
    show_kernel_threads: bool,
}

impl<'a> ProcessList<'a> {
    pub fn new(data: &'a System) -> Self {
        ProcessList {
            current_data: data,
            usernames: HashMap::default(),
            show_kernel_threads: false,
        }
    }

    pub fn show_kernel_threads(mut self, show: bool) -> Self {
        self.show_kernel_threads = show;

        self
    }
}

impl<'a> Widget for &mut ProcessList<'a> {
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
                    crate::sysinfo_thread::ProcessType::Process => Style::default().cyan(),
                    crate::sysinfo_thread::ProcessType::KernelThread => {
                        if !self.show_kernel_threads {
                            return None;
                        }
                        Style::default().gray()
                    }
                    crate::sysinfo_thread::ProcessType::Thread => Style::default(),
                };
                let user = if let Some(user) = p.user {
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
                        format!("{:>5.1}%", p.cpu_usage),
                        p.command.clone(),
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

        Table::new(rows, widths)
            .column_spacing(1)
            .header(
                Row::new(vec![
                    "PID", "User", "Name", "Virt", "Res", "S", "CPU%", "Command",
                ])
                .style(Style::new().bold()),
            )
            .highlight_style(Style::new().reversed())
            .block(Block::bordered())
            .render(area, buf);
    }
}
