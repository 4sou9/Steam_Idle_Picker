use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct IdleManager {
    processes: Mutex<HashMap<u32, Child>>,
    idler_exe: PathBuf,
}

impl IdleManager {
    pub fn new(resource_dir: PathBuf) -> Self {
        let engine_dir = resource_dir.join("engine");
        Self {
            processes: Mutex::new(HashMap::new()),
            idler_exe: engine_dir.join("steam-idle.exe"),
        }
    }

    pub fn is_idling(&self, app_id: u32) -> bool {
        let mut processes = self.processes.lock().unwrap();
        match processes.get_mut(&app_id) {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }

    pub fn get_idling_ids(&self) -> Vec<u32> {
        let mut processes = self.processes.lock().unwrap();
        processes.retain(|_, child| matches!(child.try_wait(), Ok(None)));
        processes.keys().copied().collect()
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

        self.processes.lock().unwrap().insert(app_id, child);
        true
    }

    pub fn stop_idle(&self, app_id: u32) {
        let mut processes = self.processes.lock().unwrap();
        if let Some(mut child) = processes.remove(&app_id) {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    pub fn stop_all(&self) {
        let mut processes = self.processes.lock().unwrap();
        for (_, mut child) in processes.drain() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
