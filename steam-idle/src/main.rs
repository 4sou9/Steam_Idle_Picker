// Hides the console window on Windows; steam-idle runs silently in the background.
#![windows_subsystem = "windows"]

use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    let appid: u32 = match env::args().nth(1).and_then(|s| s.parse().ok()) {
        Some(id) => id,
        None => std::process::exit(1),
    };

    // Steamworks reads SteamAppId from the environment when initializing.
    env::set_var("SteamAppId", appid.to_string());

    let (_client, single) = match steamworks::Client::init_app(appid) {
        Ok(pair) => pair,
        Err(_) => std::process::exit(1),
    };

    loop {
        single.run_callbacks();
        thread::sleep(Duration::from_secs(1));
    }
}
