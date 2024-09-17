use std::{collections::HashMap, io, sync::mpsc, thread, time::Duration};

use procfs::{
    process::{self, Process, ProcessesIter, Stat, Task},
    CpuTime, Current, CurrentSI, KernelStats, LoadAverage, Meminfo, Uptime,
};

use crate::Message;

pub fn start_thread(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> io::Result<()> {
    thread::Builder::new()
        .name("fell-sysinfo".to_string())
        .spawn(move || thread_main(tx, rx))?;

    Ok(())
}

fn thread_main(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) {
    let page_size = procfs::page_size();
    let ticks_per_sec = procfs::ticks_per_second();
    let mut procstats: HashMap<i32, ProcStats> = HashMap::new();
    let mut cpu_total_prev = CpuMetrics::default();
    let mut cpus_prev: Vec<CpuMetrics> = Vec::new();
    let mut send_threads = false;

    loop {
        let uptime = if let Ok(current) = Uptime::current() {
            current.uptime
        } else {
            break;
        };

        let (processes, thread_count) = if let Ok(processes) = process::all_processes() {
            convert_to_process_infos(
                processes,
                send_threads,
                page_size,
                uptime,
                ticks_per_sec,
                &mut procstats,
            )
        } else {
            (Vec::default(), ThreadCount::default())
        };

        let load_avg = if let Ok(current) = LoadAverage::current() {
            LoadAvg::from_load_average(&current)
        } else {
            LoadAvg::default()
        };

        let (average_cpu, cpu_percents) = if let Ok(current) = KernelStats::current() {
            let metrics = CpuMetrics::from(&current.total);
            let cpus: Vec<CpuMetrics> = current.cpu_time.iter().map(CpuMetrics::from).collect();

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

        let mem_info = if let Ok(meminfo) = Meminfo::current() {
            meminfo.into()
        } else {
            MemSwapInfo::default()
        };

        if tx
            .send(Message::SysInfo(System::new(
                processes,
                thread_count,
                uptime,
                load_avg,
                average_cpu,
                cpu_percents,
                mem_info,
            )))
            .is_err()
        {
            break;
        }

        procstats.retain(|_, p| p.last_update.eq(&uptime));

        if let Ok(Message::SendThreads(state)) = rx.recv_timeout(Duration::from_millis(1_500)) {
            if send_threads != state {
                procstats.clear();
                send_threads = state;
            }
        }
    }
}

fn convert_to_process_infos(
    processes: ProcessesIter,
    send_threads: bool,
    page_size: u64,
    uptime: f64,
    ticks_per_sec: u64,
    procstats: &mut impl ProcStatsHashMap,
) -> (Vec<ProcessInfo>, ThreadCount) {
    let mut process_infos: Vec<ProcessInfo> = Vec::default();
    let mut thread_count = ThreadCount::default();

    for p in processes.flatten() {
        if !send_threads {
            let (procinfo, threads) =
                ProcessInfo::from_process(&p, page_size, uptime, ticks_per_sec, procstats);

            thread_count.threads += threads;

            match procinfo.process_type {
                ProcessType::Process => thread_count.processes += 1,
                ProcessType::KernelThread => thread_count.kernel_threads += 1,
                ProcessType::Thread => thread_count.threads += 1,
            }

            process_infos.push(procinfo);
        } else if let Ok(tasks) = p.tasks() {
            for t in tasks.flatten() {
                let procinfo =
                    ProcessInfo::from_task(&t, &p, page_size, uptime, ticks_per_sec, procstats);

                match procinfo.process_type {
                    ProcessType::Process => thread_count.processes += 1,
                    ProcessType::KernelThread => thread_count.kernel_threads += 1,
                    ProcessType::Thread => thread_count.threads += 1,
                }

                process_infos.push(procinfo);
            }
        }
    }

    (process_infos, thread_count)
}

#[derive(Debug, Default)]
pub struct ThreadCount {
    pub processes: i64,
    pub kernel_threads: i64,
    pub threads: i64,
}

#[derive(Debug, Default)]
pub struct MemSwapInfo {
    pub mem_total: u64,
    pub mem_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl From<Meminfo> for MemSwapInfo {
    fn from(value: Meminfo) -> Self {
        MemSwapInfo {
            mem_total: value.mem_total,
            mem_used: value.mem_total - value.mem_free,
            swap_total: value.swap_total,
            swap_used: value.swap_total - value.swap_free,
        }
    }
}

#[derive(Debug, Default)]
pub struct System {
    pub processes: Vec<ProcessInfo>,
    pub thread_count: ThreadCount,
    pub uptime: Duration,
    pub load_avg: LoadAvg,
    pub average_cpu: Option<f64>,
    pub cpu_percents: Option<Vec<f64>>,
    pub mem_info: MemSwapInfo,
}

impl System {
    fn new(
        processes: Vec<ProcessInfo>,
        thread_count: ThreadCount,
        uptime: f64,
        load_avg: LoadAvg,
        average_cpu: Option<f64>,
        cpu_percents: Option<Vec<f64>>,
        mem_info: MemSwapInfo,
    ) -> Self {
        System {
            processes,
            thread_count,
            uptime: Duration::from_secs(uptime as u64),
            load_avg,
            average_cpu,
            cpu_percents,
            mem_info,
        }
    }
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

impl ProcessInfo {
    fn from_task(
        t: &Task,
        p: &Process,
        page_size: u64,
        uptime: f64,
        ticks_per_sec: u64,
        procstats: &mut impl ProcStatsHashMap,
    ) -> Self {
        let (name, cpu_usage, memory, virtual_memory, state) = if let Ok(stat) = t.stat() {
            let cpu_usage = procstats.cpu_usage(t.tid, &stat, uptime, ticks_per_sec);
            let memory = stat.rss * page_size;
            let virtual_memory = stat.vsize;
            (stat.comm, cpu_usage, memory, virtual_memory, stat.state)
        } else {
            (String::default(), 0.0, 0, 0, ' ')
        };

        let (command, process_type) = if let Ok(cmd) = p.cmdline() {
            let process_type = if cmd.is_empty() {
                ProcessType::KernelThread
            } else if p.pid == t.tid {
                ProcessType::Process
            } else {
                ProcessType::Thread
            };

            (cmd.join(" "), process_type)
        } else {
            (String::default(), ProcessType::Process)
        };

        ProcessInfo {
            pid: t.tid,
            name,
            memory,
            virtual_memory,
            cpu_usage,
            user: p.uid().ok(),
            command,
            process_type,
            state,
        }
    }

    fn from_process(
        p: &Process,
        page_size: u64,
        uptime: f64,
        ticks_per_sec: u64,
        procstats: &mut impl ProcStatsHashMap,
    ) -> (Self, i64) {
        let (name, cpu_usage, memory, virtual_memory, state, threads) = if let Ok(stat) = p.stat() {
            let cpu_usage = procstats.cpu_usage(p.pid, &stat, uptime, ticks_per_sec);

            let memory = stat.rss * page_size;
            let virtual_memory = stat.vsize;
            (
                stat.comm,
                cpu_usage,
                memory,
                virtual_memory,
                stat.state,
                stat.num_threads - 1,
            )
        } else {
            (String::default(), 0.0, 0, 0, ' ', 0)
        };

        let (command, process_type) = if let Ok(cmd) = p.cmdline() {
            let process_type = if cmd.is_empty() {
                ProcessType::KernelThread
            } else {
                ProcessType::Process
            };

            (cmd.join(" "), process_type)
        } else {
            (String::default(), ProcessType::Process)
        };

        (
            ProcessInfo {
                pid: p.pid(),
                name,
                memory,
                virtual_memory,
                cpu_usage,
                user: p.uid().ok(),
                command,
                process_type,
                state,
            },
            threads,
        )
    }
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

impl From<&CpuTime> for CpuMetrics {
    fn from(value: &CpuTime) -> Self {
        CpuMetrics {
            user: value.user,
            system: value.system,
            nice: value.nice,
            idle: value.idle,
            iowait: value.iowait,
            irq: value.irq,
            softirq: value.softirq,
            steal: value.steal,
            guest: value.guest,
            guest_nice: value.guest_nice,
        }
    }
}

impl CpuMetrics {
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
