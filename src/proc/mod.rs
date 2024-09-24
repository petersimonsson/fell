use std::{fmt::Display, fs};

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read from /proc")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse stat")]
    StatParsing,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Debug)]
pub struct System {
    processes: Vec<ProcessInfo>,
}

#[derive(Default, Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub memory: u64,
    pub virtual_memory: u64,
    pub stat: Stat,
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
                    let cmdline = fs::read_to_string(entry.path().join("cmdline"))
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

                    processes.push(ProcessInfo {
                        pid,
                        name: stat.name.clone(),
                        memory: stat.memory_res,
                        virtual_memory: stat.memory_virtual,
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

#[derive(Parser)]
#[grammar = "proc/stat.pest"]
struct StatParser;

#[derive(Default, Debug)]
struct Stat {
    name: String,
    memory_res: u64,
    memory_virtual: u64,
    state: State,
    cpu_used: u32,
    num_threads: u32,
}

#[derive(Default, Debug)]
enum State {
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

impl Stat {
    fn parse(value: &str) -> Result<Self> {
        let mut record = StatParser::parse(Rule::record, value)
            .map_err(|_| Error::StatParsing)?
            .next()
            .unwrap()
            .into_inner();

        record.next().ok_or(Error::StatParsing)?;

        let name = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .to_string();

        let state: State = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .into();

        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;

        let utime: u32 = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing)?;
        let stime: u32 = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing)?;

        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;

        let num_threads: u32 = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing)?;

        record.next().ok_or(Error::StatParsing)?;
        record.next().ok_or(Error::StatParsing)?;

        let memory_virtual: u64 = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing)?;
        let memory_res: u64 = record
            .next()
            .ok_or(Error::StatParsing)?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing)?;

        Ok(Stat {
            name,
            state,
            memory_res,
            memory_virtual,
            cpu_used: utime + stime,
            num_threads,
        })
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
