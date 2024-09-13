use std::sync::mpsc;

use app::App;
use crossterm::event::Event;
use sysinfo_thread::System;

mod app;
mod cpu_info_widget;
mod event;
mod process_list;
mod sysinfo_thread;
mod system_info_widget;
mod tui;
mod utils;

pub enum Message {
    SysInfo(System),
    Event(Event),
    SendThreads(bool),
}

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init()?;
    let (thread_tx, thread_rx) = mpsc::channel::<Message>();
    let (main_tx, main_rx) = mpsc::channel::<Message>();
    sysinfo_thread::start_thread(thread_tx.clone(), main_rx)?;
    event::start_thread(thread_tx)?;
    let app_result = App::default().run(&mut terminal, thread_rx, main_tx);
    tui::restore()?;
    Ok(app_result?)
}
