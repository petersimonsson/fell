use std::{fs, path::Path, str::FromStr};

use super::{prev_cpu::PrevCpuMap, stat::Stat, state::State, Proc, Result};

#[derive(Default, Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub uid: Option<u32>,
    pub name: String,
    pub state: State,
    pub memory: usize,
    pub virtual_memory: usize,
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

impl ProcessInfo {
    pub(super) fn read(
        proc: &mut Proc,
        pid: i32,
        parent: i32,
        path: &Path,
        uptime: f64,
    ) -> Result<Option<Self>> {
        if let Ok(stat) = fs::read_to_string(path.join("stat")) {
            let uid = if let Ok(stat) = rustix::fs::stat(path) {
                Some(stat.st_uid)
            } else {
                None
            };
            let cmdline = fs::read_to_string(path.join("cmdline"))
                .unwrap_or_default()
                .replace('\0', " ")
                .trim()
                .to_string();

            let stat = Stat::from_str(&stat)?;

            let process_type = if cmdline.is_empty() {
                ProcessType::KernelThread
            } else if pid == parent {
                ProcessType::Task
            } else {
                ProcessType::Thread
            };

            Ok(Some(ProcessInfo {
                pid,
                uid,
                name: stat.name,
                state: stat.state,
                memory: stat.memory_res * proc.page_size,
                virtual_memory: stat.memory_virtual,
                cpu_usage: proc
                    .prev_cpus
                    .calculate(pid, uptime, stat.cpu_used, proc.ticks),
                cmdline,
                process_type,
                num_threads: stat.num_threads,
            }))
        } else {
            Ok(None)
        }
    }
}
