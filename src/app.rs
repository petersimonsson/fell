use std::io;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{select, FutureExt, StreamExt};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    widgets::{Block, Borders, Row, Table, Widget},
    Frame,
};
use tokio::{pin, sync::mpsc};

use crate::{
    sysinfo_thread::{Message, ProcessInfo},
    tui::Tui,
};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    processes: Vec<ProcessInfo>,
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
        frame.render_widget(self, frame.size());
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
        let mut processes = msg.processes;
        processes.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap().reverse());
        self.processes = processes;
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let widths = [
            Constraint::Length(6),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(6),
        ];

        let rows: Vec<Row> = self
            .processes
            .iter()
            .map(|p| {
                Row::new(vec![
                    p.pid.to_string(),
                    p.name.clone(),
                    p.virtual_memory.to_string(),
                    p.memory.to_string(),
                    p.cpu_usage.to_string(),
                ])
            })
            .collect();

        let process_table = Table::new(rows, widths)
            .column_spacing(1)
            .block(Block::new().title("Processes").borders(Borders::ALL))
            .header(Row::new(vec!["PID", "Name", "Virt", "Res", "CPU%"]).style(Style::new().bold()))
            .highlight_style(Style::new().reversed());

        process_table.render(area, buf);
    }
}
