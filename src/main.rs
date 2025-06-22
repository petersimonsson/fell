use std::sync::mpsc;

use app::App;
use crossterm::event::Event;
use proc::System;

mod app;
mod cpu_info_widget;
mod event;
mod proc;
mod process_list;
mod sysinfo_thread;
mod system_info_widget;
mod utils;

pub enum Message {
    SysInfo(System),
    Event(Event),
    SendThreads(bool),
    Error(proc::Error),
}

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let (thread_tx, thread_rx) = mpsc::channel::<Message>();
    let (main_tx, main_rx) = mpsc::channel::<Message>();
    sysinfo_thread::start_thread(thread_tx.clone(), main_rx)?;
    event::start_thread(thread_tx)?;
    let app_result = App::new(false, true).run(&mut terminal, thread_rx, main_tx);
    ratatui::restore();
    Ok(app_result?)
}
