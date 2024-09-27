use std::{io, sync::mpsc, thread, time::Duration};

use crate::{proc::Proc, Message};

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
        if let Ok(system) = proc.get_system(send_threads) {
            if tx.send(Message::SysInfo(system)).is_err() {
                break;
            }
        }

        if let Ok(Message::SendThreads(state)) = rx.recv_timeout(Duration::from_millis(1_500)) {
            if send_threads != state {
                send_threads = state;
                proc.reset_prev_cpus();
            }
        }
    }
}
