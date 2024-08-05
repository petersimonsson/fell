use std::{io, path::PathBuf, thread, time::Duration};

use sysinfo::{
    CpuRefreshKind, Pid, ProcessRefreshKind, ProcessStatus, RefreshKind, System, ThreadKind,
    UpdateKind, Users,
};
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
    let mut users = Users::new();
    let process_refresh = ProcessRefreshKind::new()
        .with_cpu()
        .with_memory()
        .with_exe(UpdateKind::OnlyIfNotSet)
        .with_user(UpdateKind::OnlyIfNotSet);
    let cpu_refresh = CpuRefreshKind::new().with_cpu_usage();
    let refresh = RefreshKind::new()
        .with_processes(process_refresh)
        .with_cpu(cpu_refresh);

    loop {
        sys.refresh_specifics(refresh);
        users.refresh_list();

        let mut processes: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(_, p)| {
                let user = if let Some(user_id) = p.user_id() {
                    users
                        .get_user_by_id(user_id)
                        .map(|user| user.name().to_string())
                } else {
                    None
                };
                ProcessInfo {
                    pid: p.pid(),
                    name: p.name().to_string_lossy().to_string(),
                    memory: p.memory(),
                    virtual_memory: p.virtual_memory(),
                    cpu_usage: p.cpu_usage() / sys.cpus().len() as f32,
                    thread_kind: p.thread_kind(),
                    user,
                    exe: p.exe().map(|e| e.to_owned()),
                }
            })
            .collect();
        processes.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap().reverse());

        let mut tasks = 0;
        let mut threads = 0;
        let mut kernel_threads = 0;
        let mut running = 0;

        for proc in sys.processes().values() {
            if let Some(kind) = proc.thread_kind() {
                if kind == ThreadKind::Kernel {
                    kernel_threads += 1;
                } else {
                    threads += 1;
                }
            } else {
                tasks += 1;
            }

            if proc.status() == ProcessStatus::Run {
                running += 1;
            }
        }

        if tx
            .blocking_send(Message {
                processes,
                tasks,
                threads,
                kernel_threads,
                running,
                uptime: Duration::from_secs(System::uptime()),
            })
            .is_err()
        {
            break;
        }

        thread::sleep(Duration::from_millis(1_500));
    }
}

#[derive(Debug, Default)]
pub struct Message {
    pub processes: Vec<ProcessInfo>,
    pub tasks: u64,
    pub threads: u64,
    pub kernel_threads: u64,
    pub running: u64,
    pub uptime: Duration,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub memory: u64,
    pub virtual_memory: u64,
    pub cpu_usage: f32,
    pub thread_kind: Option<ThreadKind>,
    pub user: Option<String>,
    pub exe: Option<PathBuf>,
}
