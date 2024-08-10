use std::io;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{select, FutureExt, StreamExt};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Style, Styled, Stylize},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Row, Table, Widget},
    Frame,
};
use tokio::{pin, sync::mpsc};

use crate::{sysinfo_thread::Message, tui::Tui};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    current_data: Message,
}

impl App {
    pub async fn run(
        &mut self,
        terminal: &mut Tui,
        mut thread_rx: mpsc::Receiver<Message>,
    ) -> io::Result<()> {
        let mut ev_reader = EventStream::new();

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;

            let mut event = ev_reader.next().fuse();
            let msg = thread_rx.recv().fuse();
            pin!(msg);

            select! {
                maybe_msg = msg => {
                    match maybe_msg {
                        Some(msg) => self.handle_msg(msg),
                        None => break,
                    }
                },
                maybe_event = event => {
                    match maybe_event {
                        Some(Ok(event)) => self.handle_event(event),
                        Some(Err(_)) => {},
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_msg(&mut self, msg: Message) {
        self.current_data = msg;
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vertical = Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]);
        let [info_area, process_area] = vertical.areas(area);

        let info = vec![
            Line::default().spans(vec![
                "Uptime: ".set_style(Style::default()),
                humantime::format_duration(self.current_data.uptime)
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
            .block(Block::new().padding(Padding::symmetric(2, 1)))
            .render(info_area, buf);

        let mut max_user = 0;

        let rows: Vec<Row> = self
            .current_data
            .processes
            .iter()
            .map(|p| {
                let style = if p.kernel_thread {
                    Style::default().gray()
                } else {
                    Style::default().cyan()
                };
                if let Some(user) = &p.user {
                    max_user = max_user.max(user.len());
                }
                Row::new(vec![
                    p.pid.to_string(),
                    p.user.clone().unwrap_or_default(),
                    p.name.clone(),
                    human_bytes::human_bytes(p.virtual_memory as f64),
                    human_bytes::human_bytes(p.memory as f64),
                    format!("{:.1}%", p.cpu_usage),
                    p.command.clone(),
                ])
                .style(style)
            })
            .collect();

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
            .render(process_area, buf);
    }
}
