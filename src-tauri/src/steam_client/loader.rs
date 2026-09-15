use std::ffi::{c_void, CString};
use std::path::PathBuf;

use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH};
use windows_sys::Win32::System::Registry::{
    RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
};

use super::native::NativeInterface;

type CreateInterfaceFn = unsafe extern "C" fn(*const i8, *mut i32) -> *mut c_void;

pub struct SteamLoader {
    handle: Option<HMODULE>,
    create_interface: Option<CreateInterfaceFn>,
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Reads a REG_SZ value.
fn read_reg_string(root: HKEY, subkey: &str, value: &str) -> Option<String> {
    let subkey = to_wide(subkey);
    let value = to_wide(value);
    unsafe {
        let mut size = 0u32;
        let status = RegGetValueW(
            root,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        );
        if status != ERROR_SUCCESS || size == 0 {
            return None;
        }
        let mut buf = vec![0u16; (size as usize).div_ceil(2)];
        let status = RegGetValueW(
            root,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
        );
        if status != ERROR_SUCCESS {
            return None;
        }
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..len]))
    }
}

/// Reads a REG_DWORD value.
fn read_reg_dword(root: HKEY, subkey: &str, value: &str) -> Option<u32> {
    let subkey = to_wide(subkey);
    let value = to_wide(value);
    let mut data = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            root,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            &mut data as *mut u32 as *mut c_void,
            &mut size,
        )
    };
    (status == ERROR_SUCCESS).then_some(data)
}

/// Reads the Steam install path from the registry, matching SteamLoader.cs.
pub fn get_install_path() -> Option<PathBuf> {
    read_reg_string(HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath")
        .or_else(|| read_reg_string(HKEY_LOCAL_MACHINE, r"SOFTWARE\Valve\Steam", "InstallPath"))
        .map(PathBuf::from)
}

/// Account ID (the `userdata` folder name) of the user logged in to the running Steam
/// client, or `None` when nobody is logged in.
pub fn get_active_user() -> Option<u32> {
    read_reg_dword(HKEY_CURRENT_USER, r"Software\Valve\Steam\ActiveProcess", "ActiveUser").filter(|&id| id != 0)
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

        let dll_name = if cfg!(target_pointer_width = "64") {
            "steamclient64.dll"
        } else {
            "steamclient.dll"
        };
        let wide_path = to_wide(&path.join(dll_name).to_string_lossy());

        // LOAD_WITH_ALTERED_SEARCH_PATH resolves steamclient's own dependencies
        // (tier0_s64.dll, vstdlib_s64.dll, ...) from the Steam folder, without touching
        // the process-wide DLL search path.
        let handle = unsafe { LoadLibraryExW(wide_path.as_ptr(), std::ptr::null_mut(), LOAD_WITH_ALTERED_SEARCH_PATH) };
        if handle.is_null() {
            return false;
        }

        let proc_name = CString::new("CreateInterface").unwrap();
        let Some(proc) = (unsafe { GetProcAddress(handle, proc_name.as_ptr() as *const u8) }) else {
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
