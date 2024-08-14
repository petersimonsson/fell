use std::{io, sync::mpsc, thread};

use crossterm::event;

use crate::Message;

pub fn start_thread(tx: mpsc::Sender<Message>) -> io::Result<()> {
    thread::Builder::new()
        .name("fell-event".to_string())
        .spawn(move || thread_main(tx))?;

    Ok(())
}

fn thread_main(tx: mpsc::Sender<Message>) {
    while let Ok(event) = event::read() {
        if tx.send(Message::Event(event)).is_err() {
            break;
        }
    }
}
