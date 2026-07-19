use std::path::Path;

pub fn available_disk_space(path: &Path) -> i64 {
    #[cfg(target_os = "linux")]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            let path_c = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();
            if libc::statvfs(path_c.as_ptr(), stat.as_mut_ptr()) == 0 {
                let s = stat.assume_init();
                return (s.f_bsize as i64).saturating_mul(s.f_bavail as i64);
            }
        }
        0
    }
    #[cfg(target_os = "macos")]
    {
        use std::mem::MaybeUninit;
        unsafe {
            let mut stat: MaybeUninit<libc::statfs> = MaybeUninit::uninit();
            let path_c = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();
            if libc::statfs(path_c.as_ptr(), stat.as_mut_ptr()) == 0 {
                let s = stat.assume_init();
                return (s.f_bsize as i64).saturating_mul(s.f_bavail as i64);
            }
        }
        0
    }
    #[cfg(windows)]
    {
        let path_str = path.to_string_lossy();
        let wide: Vec<u16> = path_str.encode_utf16().collect();
        unsafe {
            let mut free_bytes: i64 = 0;
            if windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) != 0
            {
                return free_bytes;
            }
        }
        0
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        let _ = path;
        i64::MAX
    }
}

pub const PRECHECK_MARGIN: u64 = 50 * 1024 * 1024; // 50 MiB safety margin

pub async fn available_space_checked(path: std::path::PathBuf) -> Option<u64> {
    Some(available_disk_space(&path).try_into().unwrap_or(u64::MAX))
}
