use std::{collections::HashMap, io, sync::mpsc, thread, time::Duration};

use procfs::{process, CpuTime, Current, CurrentSI, KernelStats, LoadAverage, Uptime};

use crate::Message;

pub fn start_thread(tx: mpsc::Sender<Message>) -> io::Result<()> {
    thread::Builder::new()
        .name("fell-sysinfo".to_string())
        .spawn(move || thread_main(tx))?;

    Ok(())
}

fn thread_main(tx: mpsc::Sender<Message>) {
    let page_size = procfs::page_size();
    let ticks_per_sec = procfs::ticks_per_second();
    let mut procstats: HashMap<i32, ProcStats> = HashMap::new();
    let mut procstatuses: HashMap<i32, ProcStatus> = HashMap::new();
    let mut cpu_total_prev = CpuMetrics::default();
    let mut cpus_prev: Vec<CpuMetrics> = Vec::new();

    loop {
        let uptime = if let Ok(current) = Uptime::current() {
            current.uptime
        } else {
            break;
        };
        let mut kernel_threads = 0;
        let mut threads = 0;

        let processes = if let Ok(processes) = process::all_processes() {
            let mut process_infos: Vec<ProcessInfo> = processes
                .filter_map(|p| {
                    if let Ok(p) = p {
                        let (cpu_usage, memory, virtual_memory) = if let Ok(stat) = p.stat() {
                            let used_time = stat.stime + stat.utime;
                            let old_stat = if let Some(stat) = procstats.get_mut(&p.pid) {
                                stat
                            } else {
                                procstats.insert(p.pid, ProcStats::default());

                                procstats.get_mut(&p.pid).unwrap()
                            };

                            let cpu_usage = if old_stat.last_update > 0.0 {
                                let interval =
                                    (uptime - old_stat.last_update) * ticks_per_sec as f64;
                                (used_time - old_stat.used_time) as f64 * 100.0 / interval
                            } else {
                                0.0
                            };

                            old_stat.last_update = uptime;
                            old_stat.used_time = used_time;
                            let memory = stat.rss * page_size;
                            let virtual_memory = stat.vsize;
                            (cpu_usage, memory, virtual_memory)
                        } else {
                            (0.0, 0, 0)
                        };

                        let (name, user) = if let Some(status) = procstatuses.get_mut(&p.pid) {
                            status.last_update = uptime;
                            (status.name.clone(), Some(status.uid))
                        } else if let Ok(status) = p.status() {
                            let stored = ProcStatus::new(uptime, status.name, status.euid);
                            procstatuses.insert(p.pid, stored.clone());
                            (stored.name, Some(stored.uid))
                        } else {
                            (String::default(), None)
                        };

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
                            user,
                            command,
                            kernel_thread,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            process_infos.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap().reverse());

            Some(process_infos)
        } else {
            None
        };

        let tasks = if let Some(infos) = &processes {
            infos.len() as u64 - kernel_threads
        } else {
            0
        };

        let load_avg = if let Ok(current) = LoadAverage::current() {
            LoadAvg::from_load_average(&current)
        } else {
            LoadAvg::default()
        };

        let (average_cpu, cpu_percents) = if let Ok(current) = KernelStats::current() {
            let metrics = CpuMetrics::from_cpu_time(&current.total);
            let cpus: Vec<CpuMetrics> = current
                .cpu_time
                .iter()
                .map(CpuMetrics::from_cpu_time)
                .collect();

            let ret = if cpu_total_prev.total_time() > 0 {
                let average_cpu = metrics.cpu_usage(&cpu_total_prev);
                let cpu_percents = cpus
                    .iter()
                    .zip(cpus_prev.iter())
                    .map(|(n, o)| n.cpu_usage(o))
                    .collect();

                (Some(average_cpu), Some(cpu_percents))
            } else {
                (None, None)
            };

            cpu_total_prev = metrics;
            cpus_prev = cpus;

            ret
        } else {
            (None, None)
        };

        if tx
            .send(Message::SysInfo(System {
                processes,
                tasks,
                threads: threads as u64,
                kernel_threads,
                uptime: Duration::from_secs(uptime as u64),
                load_avg,
                average_cpu,
                cpu_percents,
            }))
            .is_err()
        {
            break;
        }

        procstats.retain(|_, p| p.last_update.eq(&uptime));
        procstatuses.retain(|_, p| p.last_update.eq(&uptime));

        thread::sleep(Duration::from_millis(1_500));
    }
}

#[derive(Debug, Default)]
pub struct System {
    pub processes: Option<Vec<ProcessInfo>>,
    pub tasks: u64,
    pub threads: u64,
    pub kernel_threads: u64,
    pub uptime: Duration,
    pub load_avg: LoadAvg,
    pub average_cpu: Option<f64>,
    pub cpu_percents: Option<Vec<f64>>,
}

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub memory: u64,
    pub virtual_memory: u64,
    pub cpu_usage: f64,
    pub user: Option<u32>,
    pub command: String,
    pub kernel_thread: bool,
}

#[derive(Debug, Default)]
struct ProcStats {
    last_update: f64,
    used_time: u64,
}

#[derive(Debug, Default, Clone)]
struct ProcStatus {
    last_update: f64,
    uid: u32,
    name: String,
}

impl ProcStatus {
    fn new(last_update: f64, name: String, uid: u32) -> Self {
        ProcStatus {
            last_update,
            uid,
            name,
        }
    }
}

#[derive(Debug, Default)]
pub struct LoadAvg {
    pub one: f32,
    pub five: f32,
    pub fifteen: f32,
}

impl LoadAvg {
    fn from_load_average(avg: &LoadAverage) -> Self {
        LoadAvg {
            one: avg.one,
            five: avg.five,
            fifteen: avg.fifteen,
        }
    }
}

#[derive(Debug, Default)]
struct CpuMetrics {
    user: u64,
    system: u64,
    nice: u64,
    idle: u64,
    iowait: Option<u64>,
    irq: Option<u64>,
    softirq: Option<u64>,
    steal: Option<u64>,
    guest: Option<u64>,
    guest_nice: Option<u64>,
}

impl CpuMetrics {
    fn from_cpu_time(cpu_time: &CpuTime) -> Self {
        CpuMetrics {
            user: cpu_time.user,
            system: cpu_time.system,
            nice: cpu_time.nice,
            idle: cpu_time.idle,
            iowait: cpu_time.iowait,
            irq: cpu_time.irq,
            softirq: cpu_time.softirq,
            steal: cpu_time.steal,
            guest: cpu_time.guest,
            guest_nice: cpu_time.guest_nice,
        }
    }

    fn work_time(&self) -> u64 {
        self.user
            .saturating_add(self.system)
            .saturating_add(self.nice)
            .saturating_add(self.irq.unwrap_or(0))
            .saturating_add(self.softirq.unwrap_or(0))
    }

    fn total_time(&self) -> u64 {
        self.work_time()
            .saturating_add(self.idle)
            .saturating_add(self.iowait.unwrap_or(0))
            .saturating_add(self.steal.unwrap_or(0))
            .saturating_add(self.guest.unwrap_or(0))
            .saturating_add(self.guest_nice.unwrap_or(0))
    }

    fn cpu_usage(&self, old: &CpuMetrics) -> f64 {
        (self.work_time() - old.work_time()) as f64 * 100.0
            / (self.total_time() - old.total_time()) as f64
    }
}
