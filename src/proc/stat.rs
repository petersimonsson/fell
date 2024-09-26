use pest::Parser;
use pest_derive::Parser;

use super::{Error, Result, State};

#[derive(Parser)]
#[grammar = "proc/stat.pest"]
struct StatParser;

#[derive(Default, Debug)]
pub(super) struct Stat {
    pub(super) name: String,
    pub(super) memory_res: u64,
    pub(super) memory_virtual: u64,
    pub(super) state: State,
    pub(super) cpu_used: u32,
    pub(super) num_threads: u32,
}

impl Stat {
    pub(super) fn parse(value: &str) -> Result<Self> {
        let mut record = StatParser::parse(Rule::record, value)
            .map_err(|_| Error::StatParsing("Failed to parse values from string".to_string()))?
            .next()
            .unwrap()
            .into_inner();

        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip pid".to_string()))?;

        let name = record
            .next()
            .ok_or(Error::StatParsing("Failed to read comm".to_string()))?
            .into_inner()
            .as_str()
            .to_string();

        let state: State = record
            .next()
            .ok_or(Error::StatParsing("Failed to read state".to_string()))?
            .into_inner()
            .as_str()
            .into();

        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip ppid".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip pgrp".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip session".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip tty_nr".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip tpgid".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip flags".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip minflt".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip cminflt".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip majflt".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip cmajflt".to_string()))?;

        let utime: u32 = record
            .next()
            .ok_or(Error::StatParsing("Failed to read utime".to_string()))?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing("Failed to parse utime to u32".to_string()))?;
        let stime: u32 = record
            .next()
            .ok_or(Error::StatParsing("Failed to read stime".to_string()))?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing("Failed to parse stime to u32".to_string()))?;

        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip cutime".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip cstime".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip priority".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip nice".to_string()))?;

        let num_threads: u32 = record
            .next()
            .ok_or(Error::StatParsing("Failed to read num_threads".to_string()))?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing("Failed to parse num_threads to u32".to_string()))?;

        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip itrealvalue".to_string()))?;
        record
            .next()
            .ok_or(Error::StatParsing("Failed to skip starttime".to_string()))?;

        let memory_virtual: u64 = record
            .next()
            .ok_or(Error::StatParsing("Failed to read vsize".to_string()))?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing("Failed to parse vsize to u64".to_string()))?;
        let memory_res: u64 = record
            .next()
            .ok_or(Error::StatParsing("Failed to read rss".to_string()))?
            .into_inner()
            .as_str()
            .parse()
            .map_err(|_| Error::StatParsing("Failed to parse rss to u64".to_string()))?;

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
