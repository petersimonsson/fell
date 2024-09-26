use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use super::{Error, Result};

#[derive(Parser)]
#[grammar = "proc/cputime.pest"]
struct CpuTimeParser;

pub(super) fn parse_cpu_times(input: &str) -> Result<Vec<CpuTime>> {
    let mut file = CpuTimeParser::parse(Rule::file, input)
        .map_err(|_| Error::CpuTime("Failed to parse CPU time".to_string()))?;
    let mut ret = Vec::default();

    if let Some(file) = file.next() {
        for line in file.into_inner() {
            let cpu_time = CpuTime::from_pair(line)?;
            ret.push(cpu_time);
        }
    }

    Ok(ret)
}

#[derive(Debug, Default)]
pub(super) struct CpuTime {
    pub(super) user: u64,
    pub(super) nice: u64,
    pub(super) system: u64,
    pub(super) idle: u64,
    pub(super) iowait: u64,
    pub(super) irq: u64,
    pub(super) softirq: u64,
    pub(super) steal: u64,
    pub(super) guest: u64,
    pub(super) guest_nice: u64,
}

impl CpuTime {
    fn work(&self) -> u64 {
        self.user
            .saturating_add(self.system)
            .saturating_add(self.nice)
            .saturating_add(self.irq)
            .saturating_add(self.softirq)
    }

    fn total(&self) -> u64 {
        self.work()
            .saturating_add(self.idle)
            .saturating_add(self.iowait)
            .saturating_add(self.irq)
            .saturating_add(self.softirq)
            .saturating_add(self.steal)
            .saturating_add(self.guest)
            .saturating_add(self.guest_nice)
    }

    pub(super) fn cpu_usage(&self, old: &CpuTime) -> f32 {
        (self.work() - old.work()) as f32 * 100.0 / (self.total() - old.total()) as f32
    }

    fn from_pair(value: Pair<Rule>) -> Result<Self> {
        let mut fields = value.into_inner();
        fields.next().unwrap();
        let user: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read user time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse user time".to_string()))?;
        let nice: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read nice time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse nice time".to_string()))?;
        let system: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read system time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse system time".to_string()))?;
        let idle: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read idle time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse idle time".to_string()))?;
        let iowait: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read iowait time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse iowait time".to_string()))?;
        let irq: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read irq time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse irq time".to_string()))?;
        let softirq: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read softirq time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse softirq time".to_string()))?;
        let steal: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read steal time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse steal time".to_string()))?;
        let guest: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read guest time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse guest time".to_string()))?;
        let guest_nice: u64 = fields
            .next()
            .ok_or_else(|| Error::CpuTime("Failed to read guest_nice time".to_string()))?
            .as_str()
            .parse()
            .map_err(|_| Error::CpuTime("Failed to parse guest_nice time".to_string()))?;

        Ok(CpuTime {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
        })
    }
}
