use std::fs;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read from /proc")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Debug)]
pub struct System {
    processes: Vec<ProcessInfo>,
}

#[derive(Default, Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub stat: String,
    pub cmdline: String,
    pub process_type: ProcessType,
}

#[derive(Default, Debug)]
pub enum ProcessType {
    #[default]
    Task,
    Thread,
    KernelThread,
}

pub fn get_system() -> Result<System> {
    let dir_iter = fs::read_dir("/proc")?;
    let mut processes = Vec::new();

    for entry in dir_iter.flatten() {
        if let Ok(name) = entry.file_name().into_string() {
            if let Ok(pid) = name.parse::<i32>() {
                if let Ok(stat) = fs::read_to_string(entry.path().join("stat")) {
                    let cmdline =
                        fs::read_to_string(entry.path().join("cmdline")).unwrap_or_default();

                    let process_type = if cmdline.is_empty() {
                        ProcessType::KernelThread
                    } else {
                        ProcessType::Task
                    };

                    processes.push(ProcessInfo {
                        pid,
                        stat,
                        cmdline,
                        process_type,
                    });
                }
            }
        }
    }

    Ok(System { processes })
}

struct Stat {
    name: String,
    memory_res: u64,
    memory_virtual: u64,
    state: char,
    cpu_used: u64,
}

impl From<String> for Stat {
    fn from(value: String) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_pids() -> Result<()> {
        let system = get_system()?;

        println!("{:?}", system);

        Ok(())
    }
}
