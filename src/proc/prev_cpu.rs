use std::collections::HashMap;

pub(super) struct PrevCpu {
    pub(super) uptime: f64,
    pub(super) cpu_used: u64,
}

pub(super) trait PrevCpuMap {
    fn calculate(&mut self, pid: i32, uptime: f64, cpu_used: u64, ticks: u64) -> Option<f32>;
    fn cleanup(&mut self, uptime: f64);
}

impl PrevCpuMap for HashMap<i32, PrevCpu> {
    fn calculate(&mut self, pid: i32, uptime: f64, cpu_used: u64, ticks: u64) -> Option<f32> {
        if let Some(prev_cpu) = self.get_mut(&pid) {
            let cpu_usage = (cpu_used - prev_cpu.cpu_used) as f64 * 100.0
                / ((uptime - prev_cpu.uptime) * ticks as f64);
            prev_cpu.uptime = uptime;
            prev_cpu.cpu_used = cpu_used;

            Some(cpu_usage as f32)
        } else {
            self.insert(pid, PrevCpu { uptime, cpu_used });

            None
        }
    }

    fn cleanup(&mut self, uptime: f64) {
        self.retain(|_, p| p.uptime.eq(&uptime));
    }
}
