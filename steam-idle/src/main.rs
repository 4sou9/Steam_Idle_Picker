// Hides the console window on Windows; steam-idle runs silently in the background.
#![windows_subsystem = "windows"]

use std::env;
use std::thread;
use std::time::Duration;

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

    // Fails when Steam is not running or the account does not own the app.
    // `init_app` sets SteamAppId in the environment itself.
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
        // If the process list cannot be read, keep idling rather than stopping by mistake.
        if !win_process::is_running("steam.exe").unwrap_or(true) {
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
