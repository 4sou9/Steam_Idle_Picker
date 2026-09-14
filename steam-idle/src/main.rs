// Hides the console window on Windows; steam-idle runs silently in the background.
#![windows_subsystem = "windows"]

use std::env;
use std::thread;
use std::time::Duration;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};

/// Exit codes read by the app (src-tauri/src/idle_manager.rs).
const EXIT_INIT_FAILED: i32 = 1;
const EXIT_STEAM_CLOSED: i32 = 2;

/// How often to check that Steam is still running.
const STEAM_CHECK_INTERVAL: Duration = Duration::from_secs(5);

fn main() {
    let appid: u32 = match env::args().nth(1).and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => std::process::exit(EXIT_INIT_FAILED),
    };

    // Steamworks reads SteamAppId from the environment when initializing.
    env::set_var("SteamAppId", appid.to_string());

    // Fails when Steam is not running or the account does not own the app.
    let (_client, single) = match steamworks::Client::init_app(appid) {
        Ok(pair) => pair,
        Err(_) => std::process::exit(EXIT_INIT_FAILED),
    };

    // Once Steam exits this session is dead: a restarted Steam does not pick it up
    // again, so the app would show a game as idling that Steam does not. The check runs
    // on its own thread and terminates the process outright, because after Steam dies
    // the Steamworks calls below (and a normal exit's DLL teardown) can block.
    thread::spawn(|| loop {
        thread::sleep(STEAM_CHECK_INTERVAL);
        if !is_steam_running() {
            unsafe {
                TerminateProcess(GetCurrentProcess(), EXIT_STEAM_CLOSED as u32);
            }
        }
    });

    loop {
        single.run_callbacks();
        thread::sleep(Duration::from_secs(1));
    }
}

fn is_steam_running() -> bool {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            // Cannot tell; keep idling rather than stopping by mistake.
            return true;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut ok = Process32FirstW(snapshot, &mut entry) != 0;
        while ok {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..len]).eq_ignore_ascii_case("steam.exe") {
                found = true;
                break;
            }
            ok = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
        found
    }
}
