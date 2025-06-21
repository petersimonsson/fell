use std::str::FromStr;

use super::{Error, Result};

pub(super) fn parse_cpu_times(input: &str) -> Result<Vec<CpuTime>> {
    let mut ret = Vec::default();

    for line in input.split('\n') {
        if line.starts_with("cpu") {
            let (_, times) = line
                .split_once(' ')
                .ok_or(Error::CpuTime("Failed to parse CPU time".to_string()))?;
            let cpu_time = CpuTime::from_str(times.trim())?;
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

impl FromStr for CpuTime {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let split = s.split(' ');
        let mut cputime = Self::default();

        for (i, val) in split.enumerate() {
            match i {
                0 => {
                    cputime.user = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read user time".to_string()))?
                }
                1 => {
                    cputime.nice = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read nice time".to_string()))?
                }
                2 => {
                    cputime.system = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read system time".to_string()))?
                }
                3 => {
                    cputime.idle = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read idle time".to_string()))?
                }
                4 => {
                    cputime.iowait = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read iowait time".to_string()))?
                }
                5 => {
                    cputime.irq = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read irq time".to_string()))?
                }
                6 => {
                    cputime.softirq = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read softirq time".to_string()))?
                }
                7 => {
                    cputime.steal = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read steal time".to_string()))?
                }
                8 => {
                    cputime.guest = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read guest time".to_string()))?
                }
                9 => {
                    cputime.guest_nice = val
                        .parse()
                        .map_err(|_| Error::CpuTime("Failed to read guest nice time".to_string()))?
                }
                _ => {}
            }
        }

        Ok(cputime)
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_cputime_fromstr() {
        let line = "288589 431 105679 37547659 1653040 34363 16790 0 0 0";

        let parsed = CpuTime::from_str(line);

        println!("{:?}", parsed);
    }
}
