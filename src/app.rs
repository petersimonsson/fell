use std::sync::mpsc;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use thiserror::Error;

use crate::{
    cpu_info_widget::CpuInfoWidget,
    proc::{self, System},
    process_list::ProcessList,
    system_info_widget::SystemInfoWidget,
    Message,
};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    stopped: bool,
    show_kernel_threads: bool,
    show_threads: bool,
    current_data: System,
    error_str: String,

    main_tx: Option<mpsc::Sender<Message>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ProcError(#[from] proc::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SendError(#[from] std::sync::mpsc::SendError<Message>),
}

type Result<T> = std::result::Result<T, Error>;

impl App {
    pub fn new(show_kernel_threads: bool, show_threads: bool) -> Self {
        App {
            show_kernel_threads,
            show_threads,
            ..Default::default()
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
        thread_rx: mpsc::Receiver<Message>,
        main_tx: mpsc::Sender<Message>,
    ) -> Result<()> {
        main_tx.send(Message::SendThreads(self.show_threads))?;
        self.main_tx = Some(main_tx);

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;

            match thread_rx.recv() {
                Ok(msg) => match msg {
                    Message::SysInfo(system) => {
                        self.error_str.clear();
                        self.handle_msg(system);
                    }
                    Message::Event(event) => self.handle_event(event),
                    Message::Error(error) => self.error_str = error.to_string(),
                    _ => {}
                },
                Err(_) => break,
            }
        }

        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
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
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('k') => self.toggle_kernel_threads(),
            KeyCode::Char('t') => self.toggle_threads(),
            KeyCode::Char('p') => self.toggle_stopped(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn toggle_stopped(&mut self) {
        self.stopped = !self.stopped;
    }

    fn handle_msg(&mut self, msg: System) {
        if !self.stopped {
            self.current_data = msg;
            self.current_data
                .processes
                .sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap().reverse());
        }
    }

    fn toggle_kernel_threads(&mut self) {
        self.show_kernel_threads = !self.show_kernel_threads;
    }

    fn toggle_threads(&mut self) {
        self.show_threads = !self.show_threads;

        if let Some(tx) = &self.main_tx {
            let _ = tx.send(Message::SendThreads(self.show_threads));
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut cpu_info = CpuInfoWidget::new(&self.current_data, area.width - 47);

        let vertical = Layout::vertical([
            Constraint::Length(cpu_info.row_count().max(5) + 1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ]);
        let [info_area, process_area, infobar_area] = vertical.areas(area);

        let info_horiz =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(cpu_info.width())]);
        let [info_area, cpu_area] = info_horiz.areas(info_area);

        SystemInfoWidget::new(&self.current_data).render(info_area, buf);
        cpu_info.render(cpu_area, buf);
        ProcessList::new(&self.current_data)
            .show_kernel_threads(self.show_kernel_threads)
            .render(process_area, buf);

        let infobar_layout =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [info_area, menu_area] = infobar_layout.areas(infobar_area);
        Paragraph::new(self.error_str.clone())
            .block(Block::new().borders(Borders::TOP))
            .left_aligned()
            .render(info_area, buf);
        Paragraph::new("(Q)uit - Toggle (t)threads, (k)ernel threads - (P)ause")
            .block(Block::new().borders(Borders::TOP))
            .right_aligned()
            .render(menu_area, buf);
    }
}
