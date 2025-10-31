use std::{io, path::Path};

use crate::proc::read_lines;

use super::Error;

#[derive(Debug, Default)]
pub struct MemInfo {
    pub mem_total: usize,
    pub mem_free: usize,
    pub swap_total: usize,
    pub swap_free: usize,
}

impl MemInfo {
    pub fn parse(filename: impl AsRef<Path>) -> Result<Self, Error> {
        let mut meminfo = MemInfo::default();
        let lines = read_lines(filename)?;

        for line in lines.map_while(io::Result::ok) {
            if !line.is_empty() {
                let (key, value) = line.split_once(':').ok_or(Error::MemInfo(
                    "Failed to parse memory info line".to_string(),
                ))?;

                match key {
                    "MemTotal" => {
                        meminfo.mem_total = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse MemTotal".to_string()))?
                            .parse::<usize>()
                            .map_err(|_| Error::MemInfo("Failed to parse MemTotal".to_string()))?
                            * 1024;
                    }
                    "MemFree" => {
                        meminfo.mem_free = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse MemFree".to_string()))?
                            .parse::<usize>()
                            .map_err(|_| Error::MemInfo("Failed to parse MemFree".to_string()))?
                            * 1024;
                    }
                    "SwapTotal" => {
                        meminfo.swap_total = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse SwapTotal".to_string()))?
                            .parse::<usize>()
                            .map_err(|_| Error::MemInfo("Failed to parse SwapTotal".to_string()))?
                            * 1024;
                    }
                    "SwapFree" => {
                        meminfo.swap_free = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse SwapFree".to_string()))?
                            .parse::<usize>()
                            .map_err(|_| Error::MemInfo("Failed to parse SwapFree".to_string()))?
                            * 1024;
                        break; // Stop parsing when we got all we want
                    }
                    _ => {}
                }
            }
        }
        Ok(meminfo)
    }

    pub fn mem_used(&self) -> usize {
        self.mem_total - self.mem_free
    }

    pub fn swap_used(&self) -> usize {
        self.swap_total - self.swap_free
    }
}
