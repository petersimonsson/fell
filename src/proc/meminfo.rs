use std::str::FromStr;

use super::Error;

#[derive(Debug, Default)]
pub struct MemInfo {
    pub mem_total: usize,
    pub mem_free: usize,
    pub swap_total: usize,
    pub swap_free: usize,
}

impl FromStr for MemInfo {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut meminfo = MemInfo::default();
        for line in s.split('\n') {
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
                            .parse()
                            .map_err(|_| Error::MemInfo("Failed to parse MemTotal".to_string()))?;
                    }
                    "MemFree" => {
                        meminfo.mem_total = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse MemFree".to_string()))?
                            .parse()
                            .map_err(|_| Error::MemInfo("Failed to parse MemFree".to_string()))?;
                    }
                    "SwapTotal" => {
                        meminfo.mem_total = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse SwapTotal".to_string()))?
                            .parse()
                            .map_err(|_| Error::MemInfo("Failed to parse SwapTotal".to_string()))?;
                    }
                    "SwapFree" => {
                        meminfo.mem_total = value
                            .trim()
                            .strip_suffix(" kB")
                            .ok_or(Error::MemInfo("Failed to parse SwapFree".to_string()))?
                            .parse()
                            .map_err(|_| Error::MemInfo("Failed to parse SwapFree".to_string()))?;
                        break; // Stop parsing when we got all we want
                    }
                    _ => {}
                }
            }
        }

        Ok(meminfo)
    }
}

impl MemInfo {
    pub fn mem_used(&self) -> usize {
        self.mem_total - self.mem_free
    }

    pub fn swap_used(&self) -> usize {
        self.swap_total - self.swap_free
    }
}
