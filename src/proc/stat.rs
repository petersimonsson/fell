use std::str::FromStr;

use super::{Error, State};

#[derive(Default, Debug)]
pub(super) struct Stat {
    pub(super) name: String,
    pub(super) memory_res: usize,
    pub(super) memory_virtual: usize,
    pub(super) state: State,
    pub(super) cpu_used: u64,
    pub(super) num_threads: u32,
}

impl FromStr for Stat {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut stat = Stat::default();

        let (_, tmp) = s.split_once(" (").ok_or(Error::StatParsing(
            "Failed to find start of comm".to_string(),
        ))?;
        let (name, tmp) = tmp
            .split_once(") ")
            .ok_or(Error::StatParsing("Failed to find end of comm".to_string()))?;
        stat.name = name.to_string();

        for (i, value) in tmp.split(' ').enumerate() {
            match i {
                0 => stat.state = value.into(),
                11 => {
                    stat.cpu_used = value
                        .parse()
                        .map_err(|_| Error::StatParsing("Failed to parse utime".to_string()))?
                }
                12 => {
                    stat.cpu_used += value
                        .parse::<u64>()
                        .map_err(|_| Error::StatParsing("Failed to parse stime".to_string()))?
                }
                17 => {
                    stat.num_threads = value.parse().map_err(|_| {
                        Error::StatParsing("Failed to parse num threads".to_string())
                    })?
                }
                20 => {
                    stat.memory_virtual = value.parse().map_err(|_| {
                        Error::StatParsing("Failed to parse memory virtual".to_string())
                    })?
                }
                21 => {
                    stat.memory_res = value.parse().map_err(|_| {
                        Error::StatParsing("Failed to parse memory res".to_string())
                    })?;
                    break; // Stop parsing early when we got what we need
                }
                _ => {}
            }
        }

        Ok(stat)
    }
}
