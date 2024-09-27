use pest::Parser;
use pest_derive::Parser;

use super::{Error, Result};

#[derive(Parser)]
#[grammar = "proc/meminfo.pest"]
struct MemInfoParser;

#[derive(Debug, Default)]
pub struct MemInfo {
    pub mem_total: u64,
    pub mem_free: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl MemInfo {
    pub fn parse(input: &str) -> Result<Self> {
        let mut file = MemInfoParser::parse(Rule::file, input)
            .map_err(|_| Error::MemInfo("Failed to parse meminfo".to_string()))?;

        let mut mem_total = 0;
        let mut mem_free = 0;
        let mut swap_total = 0;
        let mut swap_free = 0;

        if let Some(pairs) = file.next() {
            for pair in pairs.into_inner() {
                match pair.as_rule() {
                    Rule::memtotal => {
                        mem_total =
                            pair.into_inner().as_str().parse().map_err(|_| {
                                Error::MemInfo("Failed to parse MemTotal".to_string())
                            })?
                    }
                    Rule::memfree => {
                        mem_free =
                            pair.into_inner().as_str().parse().map_err(|_| {
                                Error::MemInfo("Failed to parse MemFree".to_string())
                            })?
                    }
                    Rule::swaptotal => {
                        swap_total =
                            pair.into_inner().as_str().parse().map_err(|_| {
                                Error::MemInfo("Failed to parse SwapTotal".to_string())
                            })?
                    }
                    Rule::swapfree => {
                        swap_free =
                            pair.into_inner().as_str().parse().map_err(|_| {
                                Error::MemInfo("Failed to parse SwapFree".to_string())
                            })?
                    }
                    _ => {}
                }
            }
        }

        Ok(MemInfo {
            mem_total,
            mem_free,
            swap_total,
            swap_free,
        })
    }

    pub fn mem_used(&self) -> u64 {
        self.mem_total - self.mem_free
    }

    pub fn swap_used(&self) -> u64 {
        self.swap_total - self.swap_free
    }
}
