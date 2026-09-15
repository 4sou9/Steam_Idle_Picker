// Process listing shared by the app and the steam-idle helper.

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub struct ProcessEntry {
    pub pid: u32,
    /// Executable file name, e.g. `steam.exe`.
    pub exe_name: String,
}

/// Every running process, or `None` when the snapshot cannot be taken.
pub fn processes() -> Option<Vec<ProcessEntry>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut list = Vec::new();
        let mut ok = Process32FirstW(snapshot, &mut entry) != 0;
        while ok {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            list.push(ProcessEntry {
                pid: entry.th32ProcessID,
                exe_name: String::from_utf16_lossy(&entry.szExeFile[..len]),
            });
            ok = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
        Some(list)
    }
}

/// Whether a process with this executable name (case-insensitive) is running, or
/// `None` when that cannot be determined. Callers pick the safe fallback.
pub fn is_running(exe_name: &str) -> Option<bool> {
    processes().map(|list| list.iter().any(|p| p.exe_name.eq_ignore_ascii_case(exe_name)))
}
