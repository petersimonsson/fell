use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::{
    proc::System,
    utils::{human_bytes, human_duration},
};

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
        let average_cpu = if let Some(cpu_usage) = &self.current_data.cpu_usage {
            if let Some(cpu_usage) = cpu_usage.first() {
                *cpu_usage
            } else {
                0.0
            }
        } else {
            0.0
        };
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
                format!(
                    "{:.2} {:.2} {:.2}",
                    self.current_data.load_avg.one,
                    self.current_data.load_avg.five,
                    self.current_data.load_avg.fifteen
                )
                .set_style(Style::default().bold()),
            ]),
            Line::default().spans(vec![
                "Memory: ".into(),
                format!(
                    "{}/{}",
                    human_bytes(self.current_data.mem_usage.mem_used(), false),
                    human_bytes(self.current_data.mem_usage.mem_total, false)
                )
                .set_style(Style::default().bold()),
                " Swap: ".into(),
                format!(
                    "{}/{}",
                    human_bytes(self.current_data.mem_usage.swap_used(), false),
                    human_bytes(self.current_data.mem_usage.swap_total, false)
                )
                .set_style(Style::default().bold()),
            ]),
            Line::default().spans(vec![
                "Tasks: ".set_style(Style::default().cyan()),
                self.current_data
                    .num_threads
                    .tasks
                    .to_string()
                    .set_style(Style::default().cyan().bold()),
                " Threads: ".set_style(Style::default()),
                self.current_data
                    .num_threads
                    .threads
                    .to_string()
                    .set_style(Style::default().bold()),
                " Kernel Threads: ".set_style(Style::default().gray()),
                self.current_data
                    .num_threads
                    .kernel_threads
                    .to_string()
                    .set_style(Style::default().gray().bold()),
            ]),
        ];
        Paragraph::new(info).render(area, buf);
    }
}
