use std::{
    ffi::{CStr, OsStr},
    mem,
    os::unix::ffi::OsStrExt,
    ptr,
    time::Duration,
};

pub fn get_username_from_uid(uid: u32) -> Option<String> {
    let mut passwd = unsafe { mem::zeroed::<libc::passwd>() };
    let mut result = ptr::null_mut();
    let mut buf = vec![0; 2048];

    loop {
        let r =
            unsafe { libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

        if r != libc::ERANGE {
            break;
        }

        let new_size = buf.len().checked_mul(2)?;
        buf.resize(new_size, 0);
    }

    if !result.is_null() {
        let username = unsafe { OsStr::from_bytes(CStr::from_ptr(passwd.pw_name).to_bytes()) };

        Some(username.to_string_lossy().to_string())
    } else {
        None
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

    if days > 0 {
        let day = if days > 1 { "days" } else { "day" };
        format!("{days} {day}, {hours:02}:{mins:02}:{secs:02}")
    } else {
        format!("{hours:02}:{mins:02}:{secs:02}")
    }
}

pub fn human_bytes(bytes: usize, fixed_width: bool) -> String {
    if bytes > 1024 {
        let (size, prefix) = if bytes > 1099511627776 {
            (bytes as f64 / 1099511627776.0, 'T')
        } else if bytes > 1073741824 {
            (bytes as f64 / 1073741824.0, 'G')
        } else if bytes > 1048576 {
            (bytes as f64 / 1048576.0, 'M')
        } else {
            (bytes as f64 / 1024.0, 'k')
        };

        if fixed_width {
            format!("{size:>7.2}{prefix}")
        } else {
            format!("{size:.2}{prefix}")
        }
    } else if fixed_width {
        format!("{bytes:>7}")
    } else {
        format!("{bytes}")
    }
}
