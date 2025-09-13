use std::{io, sync::mpsc, thread, time::Duration};

use crate::{proc::Proc, Message};

const UPDATE_INTERVAL: u64 = 2000;

pub fn start_thread(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> io::Result<()> {
    thread::Builder::new()
        .name("fell-sysinfo".to_string())
        .spawn(move || thread_main(tx, rx))?;

    Ok(())
}

fn thread_main(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) {
    let mut send_threads = false;
    let mut proc = Proc::new();

    loop {
        match proc.get_system(send_threads) {
            Ok(system) => {
                if tx.send(Message::SysInfo(system)).is_err() {
                    break;
                }
            }
            Err(e) => {
                if tx.send(Message::Error(e)).is_err() {
                    break;
                }
            }
        }

        if let Ok(Message::SendThreads(state)) =
            rx.recv_timeout(Duration::from_millis(UPDATE_INTERVAL))
        {
            if send_threads != state {
                send_threads = state;
                proc.reset_prev_cpus();
            }
        }
    }
}
