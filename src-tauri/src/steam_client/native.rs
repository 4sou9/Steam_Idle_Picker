// Raw vtable calling helpers. steamclient64.dll exposes C++ objects; the first 8 bytes
// at an object's address is a pointer to its vtable (array of function pointers), and
// each interface method takes the object pointer as an implicit first ("this") argument.
// On x86-64 Windows there is a single calling convention, so __thiscall collapses to a
// normal C function taking `this` as arg 0 — matching the layout in the original C#
// NativeWrapper<T>/NativeClass/ISteamClient018/ISteamApps001 (see SteamIdlePicker/SteamClient).

use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;

pub trait NativeInterface {
    fn from_address(address: *mut c_void) -> Self;
}

unsafe fn vtable_fn(object: *mut c_void, index: usize) -> *const c_void {
    let vtable = *(object as *const *const *const c_void);
    *vtable.add(index)
}

fn to_cstring(s: &str) -> CString {
    CString::new(s).unwrap_or_default()
}

/// Reads a NUL-terminated UTF-8 buffer, matching NativeStrings.PointerToString.
unsafe fn read_c_string(ptr: *const c_char, max_len: usize) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    if max_len == 0 || *ptr == 0 {
        return Some(String::new());
    }
    let cstr = CStr::from_ptr(ptr);
    let bytes = cstr.to_bytes();
    let len = bytes.len().min(max_len);
    Some(String::from_utf8_lossy(&bytes[..len]).into_owned())
}

pub struct ISteamClient018 {
    address: *mut c_void,
}

impl NativeInterface for ISteamClient018 {
    fn from_address(address: *mut c_void) -> Self {
        Self { address }
    }
}

impl ISteamClient018 {
    pub fn create_steam_pipe(&self) -> i32 {
        unsafe {
            let f: extern "C" fn(*mut c_void) -> i32 =
                std::mem::transmute(vtable_fn(self.address, 0));
            f(self.address)
        }
    }

    pub fn release_steam_pipe(&self, pipe: i32) -> bool {
        unsafe {
            let f: extern "C" fn(*mut c_void, i32) -> u8 =
                std::mem::transmute(vtable_fn(self.address, 1));
            f(self.address, pipe) != 0
        }
    }

    pub fn connect_to_global_user(&self, pipe: i32) -> i32 {
        unsafe {
            let f: extern "C" fn(*mut c_void, i32) -> i32 =
                std::mem::transmute(vtable_fn(self.address, 2));
            f(self.address, pipe)
        }
    }

    pub fn release_user(&self, pipe: i32, user: i32) {
        unsafe {
            let f: extern "C" fn(*mut c_void, i32, i32) =
                std::mem::transmute(vtable_fn(self.address, 4));
            f(self.address, pipe, user)
        }
    }

    fn get_i_steam_apps(&self, user: i32, pipe: i32, version: &str) -> Option<*mut c_void> {
        let version_c = to_cstring(version);
        unsafe {
            let f: extern "C" fn(*mut c_void, i32, i32, *const c_char) -> *mut c_void =
                std::mem::transmute(vtable_fn(self.address, 15));
            let addr = f(self.address, user, pipe, version_c.as_ptr());
            if addr.is_null() {
                None
            } else {
                Some(addr)
            }
        }
    }

    pub fn get_steam_apps001(&self, user: i32, pipe: i32) -> Option<ISteamApps001> {
        self.get_i_steam_apps(user, pipe, "STEAMAPPS_INTERFACE_VERSION001")
            .map(ISteamApps001::from_address)
    }
}

pub struct ISteamApps001 {
    address: *mut c_void,
}

impl NativeInterface for ISteamApps001 {
    fn from_address(address: *mut c_void) -> Self {
        Self { address }
    }
}

impl ISteamApps001 {
    pub fn get_app_data(&self, app_id: u32, key: &str) -> Option<String> {
        const VALUE_LEN: usize = 1024;
        let key_c = to_cstring(key);
        let mut buf = vec![0u8; VALUE_LEN];

        unsafe {
            let f: extern "C" fn(*mut c_void, u32, *const c_char, *mut c_char, i32) -> i32 =
                std::mem::transmute(vtable_fn(self.address, 0));
            let result = f(
                self.address,
                app_id,
                key_c.as_ptr(),
                buf.as_mut_ptr() as *mut c_char,
                VALUE_LEN as i32,
            );
            if result == 0 {
                return None;
            }
            read_c_string(buf.as_ptr() as *const c_char, VALUE_LEN)
        }
    }
}
