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
}

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init()?;
    let (tx, rx) = mpsc::channel::<Message>();
    sysinfo_thread::start_thread(tx.clone())?;
    event::start_thread(tx)?;
    let app_result = App::default().run(&mut terminal, rx);
    tui::restore()?;
    Ok(app_result?)
}
