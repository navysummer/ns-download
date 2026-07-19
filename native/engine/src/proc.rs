pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, WaitForSingleObject};
        use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            if handle == 0 { return false; }
            let ret = WaitForSingleObject(handle, 0);
            let running = ret == WAIT_TIMEOUT;
            let _ = windows_sys::Win32::Foundation::CloseHandle(handle);
            running
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        true
    }
}

pub fn find_process_by_name(name: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    #[cfg(unix)]
    {
        let output = std::process::Command::new("pgrep").arg(name).output().ok();
        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(pid) = line.trim().parse::<u32>() {
                    pids.push(pid);
                }
            }
        }
    }
    #[cfg(windows)]
    {
        let output = std::process::Command::new("tasklist")
            .args(&["/FI", &format!("IMAGENAME eq {}", name), "/FO", "CSV", "/NH"])
            .output().ok();
        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(pid_str) = line.split(',').nth(1) {
                    if let Ok(pid) = pid_str.trim_matches('"').trim().parse::<u32>() {
                        pids.push(pid);
                    }
                }
            }
        }
    }
    pids
}

use std::path::PathBuf;

pub fn current_exe_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

pub fn no_console_window(cmd: &mut std::process::Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}
