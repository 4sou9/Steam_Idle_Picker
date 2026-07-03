use std::ffi::{c_void, CString};
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, SetDllDirectoryW};

use super::native::NativeInterface;

const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 0x8;

type CreateInterfaceFn = unsafe extern "C" fn(*const i8, *mut i32) -> *mut c_void;

pub struct SteamLoader {
    handle: Option<HMODULE>,
    create_interface: Option<CreateInterfaceFn>,
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Reads the Steam install path from the registry, matching SteamLoader.cs.
pub fn get_install_path() -> Option<PathBuf> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path: Option<String> = hklm
        .open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam")
        .and_then(|k| k.get_value("InstallPath"))
        .ok()
        .or_else(|| {
            hklm.open_subkey(r"SOFTWARE\Valve\Steam")
                .and_then(|k| k.get_value("InstallPath"))
                .ok()
        });

    path.map(PathBuf::from)
}

impl SteamLoader {
    pub fn new() -> Self {
        Self {
            handle: None,
            create_interface: None,
        }
    }

    pub fn load(&mut self) -> bool {
        if self.handle.is_some() {
            return true;
        }

        let Some(path) = get_install_path() else {
            return false;
        };

        let bin_dir = path.join("bin");
        let search_path = format!("{};{}", path.display(), bin_dir.display());
        unsafe {
            let _ = SetDllDirectoryW(PCWSTR(to_wide(&search_path).as_ptr()));
        }

        let dll_name = if cfg!(target_pointer_width = "64") {
            "steamclient64.dll"
        } else {
            "steamclient.dll"
        };
        let dll_path = path.join(dll_name);
        let wide_path = to_wide(&dll_path.to_string_lossy());

        let handle = unsafe {
            LoadLibraryExW(
                PCWSTR(wide_path.as_ptr()),
                None,
                windows::Win32::System::LibraryLoader::LOAD_LIBRARY_FLAGS(
                    LOAD_WITH_ALTERED_SEARCH_PATH,
                ),
            )
        };

        let Ok(handle) = handle else {
            return false;
        };
        if handle.is_invalid() {
            return false;
        }

        let proc_name = CString::new("CreateInterface").unwrap();
        let proc = unsafe { GetProcAddress(handle, windows::core::PCSTR(proc_name.as_ptr() as *const u8)) };
        let Some(proc) = proc else {
            return false;
        };

        self.handle = Some(handle);
        self.create_interface = Some(unsafe {
            std::mem::transmute::<unsafe extern "system" fn() -> isize, CreateInterfaceFn>(proc)
        });
        true
    }

    pub fn create_interface<T: NativeInterface>(&self, version: &str) -> Option<T> {
        let create_interface = self.create_interface?;
        let version_c = CString::new(version).ok()?;
        let mut return_code: i32 = 0;
        let address = unsafe { create_interface(version_c.as_ptr(), &mut return_code) };
        if address.is_null() {
            return None;
        }
        Some(T::from_address(address))
    }
}
