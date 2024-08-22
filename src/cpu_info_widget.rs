use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::sysinfo_thread::System;

pub struct CpuInfoWidget<'a> {
    cpu_lines: Vec<Line<'a>>,
}

impl<'a> CpuInfoWidget<'a> {
    pub fn new(data: &'a System) -> Self {
        let cpu_lines: Vec<Line> = if let Some(cpu_percents) = &data.cpu_percents {
            cpu_percents
                .iter()
                .enumerate()
                .collect::<Vec<(usize, &f64)>>()
                .chunks(4)
                .map(|v| {
                    let mut line_spans = Vec::new();
                    for (i, p) in v {
                        let number_style = if **p > 75.0 {
                            Style::default().red().bold()
                        } else if **p > 50.0 {
                            Style::default().yellow().bold()
                        } else {
                            Style::default().bold()
                        };
                        line_spans.push(format!("{:3}: ", i).set_style(Style::default()));
                        line_spans.push(format!("{:5.1}% ", p).set_style(number_style));
                    }

                    Line::default().spans(line_spans)
                })
                .collect()
        } else {
            vec![Line::default().spans(["Calculating..."])]
        };

        CpuInfoWidget { cpu_lines }
    }

    pub fn row_count(&self) -> u16 {
        self.cpu_lines.len() as u16
    }
}

impl<'a> Widget for &mut CpuInfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(self.cpu_lines.clone())
            .block(Block::bordered())
            .render(area, buf);
    }
}
