use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::proc::System;

const COL_SIZE: u16 = 11;

pub struct CpuInfoWidget<'a> {
    cpu_lines: Vec<Line<'a>>,
    width: u16,
}

impl<'a> CpuInfoWidget<'a> {
    pub fn new(data: &'a System, width: u16) -> Self {
        let cols = width / COL_SIZE;
        let cpu_lines: Vec<Line> = if let Some(cpu_percents) = &data.cpu_usage {
            cpu_percents[1..cpu_percents.len()]
                .iter()
                .enumerate()
                .collect::<Vec<(usize, &f32)>>()
                .chunks(cols as usize)
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
                        line_spans.push(format!("{:3}: ", i).into());
                        line_spans.push(format!("{:5.1}% ", p).set_style(number_style));
                    }

                    Line::default().spans(line_spans)
                })
                .collect()
        } else {
            vec![Line::default().spans(["Calculating..."])]
        };

        let width = if cpu_lines.len() == 1 {
            cpu_lines.first().unwrap().width() as u16
        } else {
            cols * COL_SIZE
        };

        CpuInfoWidget { cpu_lines, width }
    }

    pub fn row_count(&self) -> u16 {
        self.cpu_lines.len() as u16
    }

    pub fn width(&self) -> u16 {
        self.width
    }
}

impl<'a> Widget for &mut CpuInfoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(self.cpu_lines.clone()).render(area, buf);
    }
}
