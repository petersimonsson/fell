use std::{collections::HashMap, io, thread, time::Duration};

use procfs::{process, Current, Uptime};
use tokio::sync::mpsc;

pub fn start_thread() -> io::Result<mpsc::Receiver<Message>> {
    let (tx, rx) = mpsc::channel(10);
    thread::Builder::new()
        .name("fell-sysinfo".to_string())
        .spawn(move || thread_main(tx))?;

    Ok(rx)
}

fn thread_main(tx: mpsc::Sender<Message>) {
    let page_size = procfs::page_size();
    let ticks_per_sec = procfs::ticks_per_second();
    let mut running_processes: HashMap<i32, ProcStats> = HashMap::new();
    loop {
        let uptime = if let Ok(current) = Uptime::current() {
            current.uptime
        } else {
            break;
        };
        let mut kernel_threads = 0;
        let mut threads = 0;

        if let Ok(processes) = process::all_processes() {
            let mut process_infos: Vec<ProcessInfo> = processes
                .filter_map(|p| {
                    if let Ok(p) = p {
                        let mut cpu_usage = 0.0;
                        let mut memory = 0;
                        let mut virtual_memory = 0;
                        if let Ok(stat) = p.stat() {
                            let used_time = stat.stime + stat.utime;
                            let process_status =
                                if let Some(status) = running_processes.get_mut(&p.pid) {
                                    status
                                } else {
                                    let status = ProcStats::default();
                                    running_processes.insert(p.pid, status);

                                    running_processes.get_mut(&p.pid).unwrap()
                                };

                            cpu_usage = if process_status.last_update > 0.0 {
                                let interval =
                                    (uptime - process_status.last_update) * ticks_per_sec as f64;
                                (used_time - process_status.used_time) as f64 * 100.0 / interval
                            } else {
                                0.0
                            };

                            process_status.last_update = uptime;
                            process_status.used_time = used_time;
                            memory = stat.rss * page_size;
                            virtual_memory = stat.vsize;
                        };

                        let mut name = String::default();
                        if let Ok(status) = p.status() {
                            name = status.name;
                        }

                        let (command, kernel_thread) = if let Ok(cmd) = p.cmdline() {
                            if cmd.is_empty() {
                                kernel_threads += 1;
                            }

                            (cmd.join(" "), cmd.is_empty())
                        } else {
                            (String::default(), false)
                        };

                        if let Ok(tasks) = p.tasks() {
                            threads += tasks.count();
                        }

                        Some(ProcessInfo {
                            pid: p.pid(),
                            name,
                            memory,
                            virtual_memory,
                            cpu_usage,
                            user: None,
                            command,
                            kernel_thread,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            let tasks = process_infos.len() as u64 - kernel_threads;

            process_infos.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap().reverse());

            if tx
                .blocking_send(Message {
                    processes: process_infos,
                    tasks,
                    threads: threads as u64,
                    kernel_threads,
                    uptime: Duration::from_secs(uptime as u64),
                })
                .is_err()
            {
                break;
            }

            running_processes.retain(|_, p| p.last_update.eq(&uptime));
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
    pub uptime: Duration,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub memory: u64,
    pub virtual_memory: u64,
    pub cpu_usage: f64,
    pub user: Option<String>,
    pub command: String,
    pub kernel_thread: bool,
}

#[derive(Debug, Default)]
struct ProcStats {
    last_update: f64,
    used_time: u64,
}
