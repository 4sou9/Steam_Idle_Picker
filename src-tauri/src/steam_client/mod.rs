// Minimal FFI bridge into steamclient64.dll, mirroring the vtable layout used by
// SteamIdlePicker's WPF version (SteamIdlePicker/SteamClient/*.cs). Only the handful
// of interfaces needed to resolve an AppID -> game name are implemented.

pub mod loader;
mod native;

pub use loader::{get_install_path, SteamLoader};
use native::{ISteamApps001, ISteamClient018};

/// Owns a steamclient pipe/user session and the resolved ISteamApps001 interface.
pub struct SteamClientSession {
    loader: SteamLoader,
    client: Option<ISteamClient018>,
    pipe: i32,
    user: i32,
    apps001: Option<ISteamApps001>,
}

impl SteamClientSession {
    pub fn new() -> Self {
        Self {
            loader: SteamLoader::new(),
            client: None,
            pipe: 0,
            user: 0,
            apps001: None,
        }
    }

    /// Attempts to connect to a running Steam client. Returns false (never errors)
    /// if steamclient64.dll can't be loaded or Steam isn't running.
    pub fn initialize(&mut self) -> bool {
        if !self.loader.load() {
            return false;
        }

        let client = match self.loader.create_interface::<ISteamClient018>("SteamClient018") {
            Some(c) => c,
            None => return false,
        };

        let pipe = client.create_steam_pipe();
        if pipe == 0 {
            return false;
        }

        let user = client.connect_to_global_user(pipe);
        if user == 0 {
            self.pipe = pipe;
            self.client = Some(client);
            return false;
        }

        let apps001 = client.get_steam_apps001(user, pipe);

        self.pipe = pipe;
        self.user = user;
        let ok = apps001.is_some();
        self.apps001 = apps001;
        self.client = Some(client);
        ok
    }

    pub fn get_game_name(&self, app_id: u32) -> Option<String> {
        self.apps001.as_ref()?.get_app_data(app_id, "name")
    }
}

impl Drop for SteamClientSession {
    fn drop(&mut self) {
        if let Some(client) = &self.client {
            if self.user > 0 {
                client.release_user(self.pipe, self.user);
            }
            if self.pipe > 0 {
                client.release_steam_pipe(self.pipe);
            }
        }
    }
}
