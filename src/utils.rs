use std::{ffi::CStr, mem, ptr, time::Duration};

pub fn get_username_from_uid(uid: u32) -> Option<String> {
    unsafe {
        let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
            n if n < 0 => 512_usize,
            n => n as usize,
        };
        let mut buf = Vec::with_capacity(amt);
        let mut passwd = mem::zeroed::<libc::passwd>();
        let mut result = ptr::null_mut();

        match libc::getpwuid_r(
            uid,
            &mut passwd,
            buf.as_mut_ptr(),
            buf.capacity(),
            &mut result,
        ) {
            0 if !result.is_null() => {
                let username = CStr::from_ptr(passwd.pw_name).to_string_lossy().to_string();

                Some(username)
            }
            _ => None,
        }
    }
}

pub fn human_duration(duration: Duration) -> String {
    let secs = duration.as_secs();

    let days = secs / 86400;
    let secs = secs % 86400;
    let hours = secs / 3600;
    let secs = secs % 3600;
    let mins = secs / 60;
    let secs = secs % 60;
    let day = if days > 1 { "days" } else { "day" };

    format!("{days} {day}, {hours:02}:{mins:02}:{secs:02}")
}
