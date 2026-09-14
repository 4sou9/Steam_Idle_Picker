use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use serde::Serialize;

use crate::process_guard::{self, Job};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Exit codes of steam-idle.exe (see steam-idle/src/main.rs).
const EXIT_INIT_FAILED: i32 = 1;
const EXIT_STEAM_CLOSED: i32 = 2;

/// Why a helper ended on its own. Serialized as the code the frontend localizes.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureReason {
    /// Steamworks init failed and Steam is not running.
    SteamNotRunning,
    /// Steamworks init failed while Steam is running (e.g. the game is not owned).
    LaunchFailed,
    /// The helper noticed Steam exiting and stopped.
    SteamClosed,
    /// Any other exit (crash, killed from outside).
    Exited,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdleFailure {
    pub app_id: u32,
    pub reason: FailureReason,
}

struct Helper {
    child: Child,
    started: Instant,
}

pub struct IdleManager {
    processes: Mutex<HashMap<u32, Helper>>,
    /// Helpers that ended on their own since the frontend last asked.
    failures: Mutex<Vec<IdleFailure>>,
    idler_exe: PathBuf,
    /// Terminates the helpers when the app dies without running `stop_all`.
    job: Option<Job>,
}

impl IdleManager {
    pub fn new(resource_dir: PathBuf) -> Self {
        let engine_dir = resource_dir.join("engine");
        Self {
            processes: Mutex::new(HashMap::new()),
            failures: Mutex::new(Vec::new()),
            idler_exe: engine_dir.join("steam-idle.exe"),
            job: Job::new(),
        }
    }

    /// Helpers from an earlier session cannot be tracked or stopped; end them so
    /// Steam stops showing those games. Safe at startup thanks to single-instance.
    pub fn kill_orphans(&self) {
        process_guard::kill_orphans(&self.idler_exe);
    }

    /// Drops helpers that have exited and records why.
    fn reap(&self, processes: &mut HashMap<u32, Helper>) {
        let mut failures = Vec::new();
        processes.retain(|&app_id, helper| {
            let Ok(Some(status)) = helper.child.try_wait() else {
                return true;
            };
            let reason = match status.code() {
                Some(EXIT_INIT_FAILED) if process_guard::is_steam_running() => FailureReason::LaunchFailed,
                Some(EXIT_INIT_FAILED) => FailureReason::SteamNotRunning,
                Some(EXIT_STEAM_CLOSED) => FailureReason::SteamClosed,
                // Older helpers exit with 1 only at startup; treat an early unknown
                // exit the same way.
                _ if helper.started.elapsed() < Duration::from_secs(5) && !process_guard::is_steam_running() => {
                    FailureReason::SteamNotRunning
                }
                _ => FailureReason::Exited,
            };
            failures.push(IdleFailure { app_id, reason });
            false
        });
        if !failures.is_empty() {
            self.failures.lock().unwrap().extend(failures);
        }
    }

    pub fn is_idling(&self, app_id: u32) -> bool {
        let mut processes = self.processes.lock().unwrap();
        self.reap(&mut processes);
        processes.contains_key(&app_id)
    }

    pub fn get_idling_ids(&self) -> Vec<u32> {
        let mut processes = self.processes.lock().unwrap();
        self.reap(&mut processes);
        processes.keys().copied().collect()
    }

    /// Failures recorded since the last call.
    pub fn take_failures(&self) -> Vec<IdleFailure> {
        std::mem::take(&mut *self.failures.lock().unwrap())
    }

    pub fn start_idle(&self, app_id: u32) -> bool {
        if self.is_idling(app_id) {
            return true;
        }
        if !self.idler_exe.is_file() {
            return false;
        }

        let engine_dir = match self.idler_exe.parent() {
            Some(dir) => dir,
            None => return false,
        };

        let mut cmd = std::process::Command::new(&self.idler_exe);
        cmd.arg(app_id.to_string()).current_dir(engine_dir);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let Ok(child) = cmd.spawn() else {
            return false;
        };
        if let Some(job) = &self.job {
            job.assign(&child);
        }

        self.processes.lock().unwrap().insert(
            app_id,
            Helper {
                child,
                started: Instant::now(),
            },
        );
        true
    }

    pub fn stop_idle(&self, app_id: u32) {
        let mut processes = self.processes.lock().unwrap();
        if let Some(mut helper) = processes.remove(&app_id) {
            let _ = helper.child.kill();
            let _ = helper.child.wait();
        }
    }

    pub fn stop_all(&self) {
        let mut processes = self.processes.lock().unwrap();
        for (_, mut helper) in processes.drain() {
            let _ = helper.child.kill();
            let _ = helper.child.wait();
        }
    }
}
