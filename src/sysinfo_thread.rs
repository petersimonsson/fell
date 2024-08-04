use std::{io, thread, time::Duration};

use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
use tokio::sync::mpsc;

pub fn start_thread() -> io::Result<mpsc::Receiver<Message>> {
    let (tx, rx) = mpsc::channel(10);
    thread::Builder::new()
        .name("fell-sysinfo".to_string())
        .spawn(move || thread_main(tx))?;

    Ok(rx)
}

fn thread_main(tx: mpsc::Sender<Message>) {
    let mut sys = System::new();
    let process_refresh = ProcessRefreshKind::new().with_cpu().with_memory();
    let refresh = RefreshKind::new().with_processes(process_refresh);

    loop {
        sys.refresh_specifics(refresh);

        let processes = sys
            .processes()
            .iter()
            .map(|(_, p)| ProcessInfo {
                pid: p.pid(),
                name: p.name().to_string_lossy().to_string(),
                memory: p.memory(),
                virtual_memory: p.virtual_memory(),
                cpu_usage: p.cpu_usage(),
            })
            .collect();

        if let Err(_) = tx.blocking_send(Message { processes }) {
            break;
        }

        thread::sleep(Duration::from_millis(1_000));
    }
}

pub struct Message {
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub memory: u64,
    pub virtual_memory: u64,
    pub cpu_usage: f32,
}
