mod cputime;
mod loadavg;
mod meminfo;
mod prev_cpu;
pub mod process_info;
mod stat;
pub mod state;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use cputime::CpuTime;
use loadavg::LoadAvg;
use meminfo::MemInfo;
use prev_cpu::{PrevCpu, PrevCpuMap};
use process_info::{ProcessInfo, ProcessType};
use state::State;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read from /proc")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse stat")]
    StatParsing(String),
    #[error("Failed to read uptime")]
    Uptime(String),
    #[error("Failed to read loadavg")]
    LoadAvg(String),
    #[error("Failed to read CPU time")]
    CpuTime(String),
    #[error("Failed to read meminfo")]
    MemInfo(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Proc {
    ticks: u64,
    page_size: usize,
    prev_cpus: HashMap<i32, PrevCpu>,
    prev_cpu_time: Vec<CpuTime>,
}

#[derive(Default, Debug)]
pub struct System {
    pub processes: Vec<ProcessInfo>,
    pub num_threads: ThreadCount,
    pub uptime: Duration,
    pub load_avg: LoadAvg,
    pub cpu_usage: Option<Vec<f32>>,
    pub mem_usage: MemInfo,
}

#[derive(Default, Debug)]
pub struct ThreadCount {
    pub tasks: u32,
    pub threads: u32,
    pub kernel_threads: u32,
}

impl Proc {
    pub fn new() -> Self {
        let ticks = rustix::param::clock_ticks_per_second();
        let page_size = rustix::param::page_size();
        Proc {
            ticks,
            page_size,
            prev_cpus: HashMap::default(),
            prev_cpu_time: Vec::default(),
        }
    }

    pub fn reset_prev_cpus(&mut self) {
        self.prev_cpus.clear();
    }

    pub fn get_system(&mut self, get_threads: bool) -> Result<System> {
        let dir_iter = fs::read_dir("/proc")?;
        let mut processes = Vec::new();
        let mut num_threads = ThreadCount::default();
        let uptime = read_uptime("/proc/uptime".into())?;

        for entry in dir_iter.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if let Ok(pid) = name.parse::<i32>() {
                    if !get_threads {
                        if let Some(info) = self.get_process_info(
                            &entry.path(),
                            pid,
                            pid,
                            uptime,
                            &mut num_threads,
                        )? {
                            processes.push(info);
                        }
                    } else {
                        let dir_iter = fs::read_dir(entry.path().join("task"))?;
                        for entry in dir_iter.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                if let Ok(tid) = name.parse::<i32>() {
                                    if let Some(info) = self.get_process_info(
                                        &entry.path(),
                                        tid,
                                        pid,
                                        uptime,
                                        &mut num_threads,
                                    )? {
                                        processes.push(info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.prev_cpus.cleanup(uptime);

        let load_avg = LoadAvg::load("/proc/loadavg".into())?;

        let input = fs::read_to_string("/proc/stat")?;
        let cpu_time = cputime::parse_cpu_times(&input)?;

        let cpu_usage = if !self.prev_cpu_time.is_empty() {
            Some(
                cpu_time
                    .iter()
                    .zip(self.prev_cpu_time.iter())
                    .map(|(new, old)| new.cpu_usage(old))
                    .collect(),
            )
        } else {
            None
        };

        let input = fs::read_to_string("/proc/meminfo")?;
        let mem_usage = MemInfo::from_str(&input)?;

        self.prev_cpu_time = cpu_time;

        Ok(System {
            processes,
            num_threads,
            uptime: Duration::from_secs(uptime as u64),
            load_avg,
            cpu_usage,
            mem_usage,
        })
    }

    fn get_process_info(
        &mut self,
        path: &Path,
        tid: i32,
        pid: i32,
        uptime: f64,
        num_threads: &mut ThreadCount,
    ) -> Result<Option<ProcessInfo>> {
        if let Some(info) = ProcessInfo::read(self, tid, pid, path, uptime)? {
            if tid == pid {
                if let ProcessType::KernelThread = info.process_type {
                    num_threads.kernel_threads += 1;
                } else {
                    num_threads.tasks += 1;
                }

                num_threads.threads += info.num_threads - 1;
            }

            Ok(Some(info))
        } else {
            Ok(None)
        }
    }
}

fn read_uptime(path: PathBuf) -> Result<f64> {
    let uptime = fs::read_to_string(&path).map_err(|_| {
        Error::Uptime(format!(
            "Could not find an uptime file at {}",
            path.display()
        ))
    })?;
    let (uptime, _) = uptime
        .split_once(' ')
        .ok_or_else(|| Error::Uptime("Failed to split string".to_string()))?;

    uptime
        .parse::<f64>()
        .map_err(|_| Error::Uptime("Failed to parse uptime to f64".to_string()))
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn get_pids() -> Result<()> {
        let mut proc = Proc::new();
        let _system = proc.get_system(true)?;
        sleep(Duration::from_millis(1_500));
        let system = proc.get_system(true)?;

        println!("{:?}", system);

        Ok(())
    }
}
