use std::{collections::HashMap, io, sync::mpsc, thread, time::Duration};

use procfs::{
    process::{self, Stat},
    CpuTime, Current, CurrentSI, KernelStats, LoadAverage, Uptime,
};

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
            let mut process_infos: Vec<ProcessInfo> = Vec::default();

            for p in processes.flatten() {
                let (name, cpu_usage, memory, virtual_memory, state) = if let Ok(stat) = p.stat() {
                    let cpu_usage = procstats.cpu_usage(p.pid, &stat, uptime, ticks_per_sec);

                    threads += stat.num_threads - 1;

                    let memory = stat.rss * page_size;
                    let virtual_memory = stat.vsize;
                    (stat.comm, cpu_usage, memory, virtual_memory, stat.state)
                } else {
                    (String::default(), 0.0, 0, 0, ' ')
                };

                let (command, process_type) = if let Ok(cmd) = p.cmdline() {
                    let process_type = if cmd.is_empty() {
                        kernel_threads += 1;
                        ProcessType::KernelThread
                    } else {
                        ProcessType::Process
                    };

                    (cmd.join(" "), process_type)
                } else {
                    (String::default(), ProcessType::Process)
                };

                if let Ok(tasks) = p.tasks() {
                    for t in tasks.flatten() {
                        if p.pid == t.tid {
                            continue;
                        }
                        let (name, cpu_usage, memory, virtual_memory, state) =
                            if let Ok(stat) = t.stat() {
                                let cpu_usage =
                                    procstats.cpu_usage(t.tid, &stat, uptime, ticks_per_sec);
                                let memory = stat.rss * page_size;
                                let virtual_memory = stat.vsize;
                                (stat.comm, cpu_usage, memory, virtual_memory, stat.state)
                            } else {
                                (String::default(), 0.0, 0, 0, ' ')
                            };

                        let command = if let Ok(cmd) = p.cmdline() {
                            cmd.join(" ")
                        } else {
                            String::default()
                        };
                        process_infos.push(ProcessInfo {
                            pid: t.tid,
                            name,
                            memory,
                            virtual_memory,
                            cpu_usage,
                            user: p.uid().ok(),
                            command,
                            process_type: ProcessType::Thread,
                            state,
                        });
                    }
                }

                process_infos.push(ProcessInfo {
                    pid: p.pid(),
                    name,
                    memory,
                    virtual_memory,
                    cpu_usage,
                    user: p.uid().ok(),
                    command,
                    process_type,
                    state,
                });
            }

            process_infos.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap().reverse());

            Some(process_infos)
        } else {
            None
        };

        let tasks = if let Some(infos) = &processes {
            infos.len() as i64 - kernel_threads - threads
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
                threads,
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

        thread::sleep(Duration::from_millis(1_500));
    }
}

#[derive(Debug, Default)]
pub struct System {
    pub processes: Option<Vec<ProcessInfo>>,
    pub tasks: i64,
    pub threads: i64,
    pub kernel_threads: i64,
    pub uptime: Duration,
    pub load_avg: LoadAvg,
    pub average_cpu: Option<f64>,
    pub cpu_percents: Option<Vec<f64>>,
}

#[derive(Debug)]
pub enum ProcessType {
    Process,
    KernelThread,
    Thread,
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
    pub process_type: ProcessType,
    pub state: char,
}

#[derive(Debug, Default)]
struct ProcStats {
    last_update: f64,
    used_time: u64,
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

trait ProcStatsHashMap {
    fn cpu_usage(&mut self, pid: i32, stat: &Stat, uptime: f64, ticks_per_sec: u64) -> f64;
}

impl ProcStatsHashMap for HashMap<i32, ProcStats> {
    fn cpu_usage(&mut self, pid: i32, stat: &Stat, uptime: f64, ticks_per_sec: u64) -> f64 {
        let used_time = stat.stime + stat.utime;
        let old_stat = if let Some(stat) = self.get_mut(&pid) {
            stat
        } else {
            self.insert(pid, ProcStats::default());

            self.get_mut(&pid).unwrap()
        };

        let cpu_usage = if old_stat.last_update > 0.0 {
            let interval = (uptime - old_stat.last_update) * ticks_per_sec as f64;
            let time_diff = used_time - old_stat.used_time;
            time_diff as f64 * 100.0 / interval
        } else {
            0.0
        };

        old_stat.last_update = uptime;
        old_stat.used_time = used_time;

        cpu_usage
    }
}
