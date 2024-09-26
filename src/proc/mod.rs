mod stat;

use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use stat::Stat;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read from /proc")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse stat")]
    StatParsing(String),
    #[error("Failed to read uptime")]
    Uptime(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Proc {
    prev_cpus: HashMap<i32, PrevCpu>,
}

#[derive(Default, Debug)]
pub struct System {
    processes: Vec<ProcessInfo>,
    num_threads: ThreadCount,
}

#[derive(Default, Debug)]
pub struct ThreadCount {
    tasks: u32,
    threads: u32,
    kernel_threads: u32,
}

#[derive(Default, Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub state: State,
    pub memory: u64,
    pub virtual_memory: u64,
    pub cpu_usage: Option<f32>,
    pub cmdline: String,
    pub process_type: ProcessType,
    pub num_threads: u32,
}

#[derive(Default, Debug)]
pub enum ProcessType {
    #[default]
    Task,
    Thread,
    KernelThread,
}

impl Proc {
    pub fn new() -> Self {
        Proc {
            prev_cpus: HashMap::default(),
        }
    }

    pub fn get_system(&mut self) -> Result<System> {
        let dir_iter = fs::read_dir("/proc")?;
        let mut processes = Vec::new();
        let mut num_threads = ThreadCount::default();
        let uptime = read_uptime("/proc/uptime".into())?;

        for entry in dir_iter.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if let Ok(pid) = name.parse::<i32>() {
                    if let Some(info) =
                        read_process_info(pid, &entry.path(), uptime, &mut self.prev_cpus)?
                    {
                        if let ProcessType::KernelThread = info.process_type {
                            num_threads.kernel_threads += 1;
                        } else {
                            num_threads.tasks += 1;
                        }

                        num_threads.threads += info.num_threads - 1;

                        processes.push(info);
                    }
                }
            }
        }

        Ok(System {
            processes,
            num_threads,
        })
    }
}

fn read_process_info(
    pid: i32,
    path: &Path,
    uptime: f64,
    prev_cpus: &mut HashMap<i32, PrevCpu>,
) -> Result<Option<ProcessInfo>> {
    if let Ok(stat) = fs::read_to_string(path.join("stat")) {
        let cmdline = fs::read_to_string(path.join("cmdline"))
            .unwrap_or_default()
            .replace('\0', " ")
            .trim()
            .to_string();

        let process_type = if cmdline.is_empty() {
            ProcessType::KernelThread
        } else {
            ProcessType::Task
        };

        let stat = Stat::parse(&stat)?;

        Ok(Some(ProcessInfo {
            pid,
            name: stat.name,
            state: stat.state,
            memory: stat.memory_res,
            virtual_memory: stat.memory_virtual,
            cpu_usage: prev_cpus.calculate(pid, uptime, stat.cpu_used),
            cmdline,
            process_type,
            num_threads: stat.num_threads,
        }))
    } else {
        Ok(None)
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

#[derive(Default, Debug)]
pub enum State {
    #[default]
    Unknown,
    Running,
    Sleeping,
    Waiting,
    Zombie,
    Stopped,
    Tracing,
    Dead,
}

impl From<&str> for State {
    fn from(value: &str) -> Self {
        match value {
            "R" => State::Running,
            "S" => State::Sleeping,
            "D" => State::Waiting,
            "Z" => State::Zombie,
            "T" => State::Stopped,
            "t" => State::Tracing,
            "X" => State::Dead,
            _ => State::Unknown,
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Unknown => write!(f, "Unknown"),
            State::Running => write!(f, "R"),
            State::Sleeping => write!(f, "S"),
            State::Waiting => write!(f, "D"),
            State::Zombie => write!(f, "Z"),
            State::Stopped => write!(f, "T"),
            State::Tracing => write!(f, "t"),
            State::Dead => write!(f, "X"),
        }
    }
}

struct PrevCpu {
    uptime: f64,
    cpu_used: u32,
}

trait PrevCpuMap {
    fn calculate(&mut self, pid: i32, uptime: f64, cpu_used: u32) -> Option<f32>;
}

impl PrevCpuMap for HashMap<i32, PrevCpu> {
    fn calculate(&mut self, pid: i32, uptime: f64, cpu_used: u32) -> Option<f32> {
        if let Some(prev_cpu) = self.get_mut(&pid) {
            let cpu_usage = (cpu_used - prev_cpu.cpu_used) as f64 / (uptime - prev_cpu.uptime);
            prev_cpu.uptime = uptime;
            prev_cpu.cpu_used = cpu_used;

            Some(cpu_usage as f32)
        } else {
            self.insert(pid, PrevCpu { uptime, cpu_used });

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_pids() -> Result<()> {
        let mut proc = Proc::new();
        let system = proc.get_system()?;

        println!("{:?}", system);

        Ok(())
    }
}
