use std::{fs, path::PathBuf};

use super::{Error, Result};

#[derive(Debug, Default)]
pub struct LoadAvg {
    pub one: f32,
    pub five: f32,
    pub fifteen: f32,
}

impl LoadAvg {
    pub(super) fn load(path: PathBuf) -> Result<Self> {
        let loadavg = fs::read_to_string(&path).map_err(|_| {
            Error::LoadAvg(format!(
                "Could not find a loadavg file at {}",
                path.display()
            ))
        })?;
        let mut loadavg = loadavg.split(' ');

        Ok(LoadAvg {
            one: loadavg
                .next()
                .ok_or_else(|| Error::LoadAvg("Failed to read 1 minute average".to_string()))?
                .parse()
                .map_err(|_| Error::LoadAvg("Failed to parse 1 minute average".to_string()))?,
            five: loadavg
                .next()
                .ok_or_else(|| Error::LoadAvg("Failed to read 5 minute average".to_string()))?
                .parse()
                .map_err(|_| Error::LoadAvg("Failed to parse 5 minute average".to_string()))?,
            fifteen: loadavg
                .next()
                .ok_or_else(|| Error::LoadAvg("Failed to read 15 minute average".to_string()))?
                .parse()
                .map_err(|_| Error::LoadAvg("Failed to parse 15 minute average".to_string()))?,
        })
    }
}
