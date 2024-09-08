use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{sysinfo_thread::System, utils::human_duration};

pub struct SystemInfoWidget<'a> {
    current_data: &'a System,
}

impl<'a> SystemInfoWidget<'a> {
    pub fn new(data: &'a System) -> Self {
        SystemInfoWidget { current_data: data }
    }
}

impl<'a> Widget for &mut SystemInfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let average_cpu = self.current_data.average_cpu.unwrap_or(0.0);
        let average_cpu_style = if average_cpu > 75.0 {
            Style::default().red().bold()
        } else if average_cpu > 50.0 {
            Style::default().yellow().bold()
        } else {
            Style::default().bold()
        };
        let info = vec![
            Line::default().spans(vec![
                "Uptime: ".set_style(Style::default()),
                human_duration(self.current_data.uptime).set_style(Style::default().bold()),
            ]),
            Line::default().spans(vec![
                "Average CPU: ".set_style(Style::default()),
                format!("{:.1}%", average_cpu).set_style(average_cpu_style),
            ]),
            Line::default().spans(vec![
                "Load average: ".set_style(Style::default()),
                self.current_data
                    .load_avg
                    .one
                    .to_string()
                    .set_style(Style::default().bold()),
                " ".set_style(Style::default()),
                self.current_data
                    .load_avg
                    .five
                    .to_string()
                    .set_style(Style::default().bold()),
                " ".set_style(Style::default()),
                self.current_data
                    .load_avg
                    .fifteen
                    .to_string()
                    .set_style(Style::default().bold()),
            ]),
            Line::default().spans(vec![
                "Tasks: ".set_style(Style::default().cyan()),
                self.current_data
                    .tasks
                    .to_string()
                    .set_style(Style::default().cyan().bold()),
                " Threads: ".set_style(Style::default()),
                self.current_data
                    .threads
                    .to_string()
                    .set_style(Style::default().bold()),
                " Kernel Threads: ".set_style(Style::default().gray()),
                self.current_data
                    .kernel_threads
                    .to_string()
                    .set_style(Style::default().gray().bold()),
            ]),
        ];
        Paragraph::new(info)
            .block(Block::bordered())
            .render(area, buf);
    }
}
