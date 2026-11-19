//! Process liveness, injectable so tests never depend on real pids.

pub trait ProcessProbe: Send {
    /// `proc_start` is the registry's Windows FILETIME; when present it guards
    /// against pid reuse.
    fn is_alive(&self, pid: u32, proc_start: Option<u64>) -> bool;
}

/// Real probe: OpenProcess + GetExitCodeProcess + GetProcessTimes on Windows.
pub struct SystemProbe;

#[cfg(windows)]
impl ProcessProbe for SystemProbe {
    fn is_alive(&self, pid: u32, proc_start: Option<u64>) -> bool {
        use windows_sys::Win32::Foundation::{CloseHandle, FILETIME};
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        const STILL_ACTIVE: u32 = 259;
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h as usize == 0 {
                return false;
            }
            let mut code: u32 = 0;
            let mut alive = GetExitCodeProcess(h, &mut code) != 0 && code == STILL_ACTIVE;
            if alive {
                if let Some(expected) = proc_start {
                    let z = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
                    let (mut c, mut e, mut k, mut u) = (z, z, z, z);
                    if GetProcessTimes(h, &mut c, &mut e, &mut k, &mut u) != 0 {
                        let created = ((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64;
                        alive = created == expected;
                    }
                }
            }
            CloseHandle(h);
            alive
        }
    }
}

#[cfg(not(windows))]
impl ProcessProbe for SystemProbe {
    fn is_alive(&self, _pid: u32, _proc_start: Option<u64>) -> bool {
        true
    }
}
