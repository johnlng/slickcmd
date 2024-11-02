use std::{ffi::c_void, mem::size_of, path::Path};

use crate::consts::APP_TITLE;
use crate::{
    raii::AutoCloseHandle,
    win32::{self, pwsz, wsz_from_str},
};
use windows::core::GUID;
use windows::Wdk::System::Threading::PROCESSINFOCLASS;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn u8s_as_guid(bts: &[u8]) -> GUID {
    debug_assert!(bts.len() == 16);
    unsafe { *(bts.as_ptr() as *const GUID) }
}

pub fn get_working_dir(pid: u32) -> String {
    let hproc = win32::open_process(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);

    let pbi = PROCESS_BASIC_INFORMATION::default();
    let ppbi = &pbi as *const PROCESS_BASIC_INFORMATION as *mut c_void;

    let status = win32::nt_query_information_process(
        hproc,
        PROCESSINFOCLASS::default(),
        ppbi,
        size_of::<PROCESS_BASIC_INFORMATION>() as u32,
    );

    let _ach = AutoCloseHandle(hproc);
    if status.is_err() {
        //?
        return String::new();
    }

    let peb = PEB::default();
    let ppeb = &peb as *const _ as *mut c_void;
    let base_addr = pbi.PebBaseAddress as *mut c_void;
    if !win32::read_process_memory(hproc, base_addr, ppeb, size_of::<PEB>()) {
        return String::new();
    }

    //
    #[repr(C)]
    pub struct RtlUserProcessParameters {
        pub reserved1: [u8; 16],
        pub reserved2: [*mut c_void; 5],
        pub current_directory_path: UNICODE_STRING,
        pub current_directory_handle: HANDLE,
        pub dll_path: UNICODE_STRING,
        pub image_path_name: UNICODE_STRING,
        pub command_line: UNICODE_STRING,
    }

    //
    let upp: RtlUserProcessParameters = unsafe { core::mem::zeroed() };
    let pupp = &upp as *const _ as *mut c_void;
    let base_addr = peb.ProcessParameters as *mut c_void;
    let cb_size = size_of::<RtlUserProcessParameters>();
    if !win32::read_process_memory(hproc, base_addr, pupp, cb_size) {
        return String::new();
    }

    let cch = upp.current_directory_path.Length / 2;
    let mut dir_buf = vec![0u16; cch as usize];
    let base_addr = upp.current_directory_path.Buffer.0 as *mut c_void;
    let p_dir_buf = dir_buf.as_mut_ptr() as *mut c_void;
    let cb_size = (cch * 2) as usize;
    if !win32::read_process_memory(hproc, base_addr, p_dir_buf, cb_size) {
        return String::new();
    }
    String::from_utf16_lossy(&dir_buf)
}

pub fn dir_exists(dir: &str) -> bool {
    let attr = win32::get_file_attributes(dir);
    if attr == INVALID_FILE_ATTRIBUTES {
        return false;
    }
    if (attr & FILE_ATTRIBUTE_DIRECTORY.0) == 0 {
        return false;
    }
    true
}

pub fn get_home_dir() -> String {
    const BUF_SIZE: usize = MAX_PATH as _;
    let mut buf = vec![0u16; BUF_SIZE];

    let mut dir = String::new();
    let cch = win32::get_environment_variable("HOMEDRIVE", &mut buf);
    if cch == 0 {
        return String::new();
    }
    dir.push_str(&String::from_utf16_lossy(&buf[..cch as usize]));

    let cch = win32::get_environment_variable("HOMEPATH", &mut buf);
    if cch == 0 {
        return String::new();
    }
    dir.push_str(&String::from_utf16_lossy(&buf[..cch as usize]));

    dir = normalize_dir_path(&dir);
    dir
}

pub fn normalize_dir_path(path: &str) -> String {
    let hfile = win32::create_file(
        path,
        GENERIC_READ.0,
        FILE_SHARE_READ,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
    );

    if hfile.is_invalid() {
        //?
        return path.into();
    }
    const BUF_SIZE: usize = MAX_PATH as _;
    let mut buf = [0u16; BUF_SIZE];
    let cch = win32::get_final_path_name_by_handle(hfile, &mut buf[..], VOLUME_NAME_DOS);
    win32::close_handle(hfile);

    if cch == 0 {
        return path.into();
    }

    let mut cch = cch as usize;
    if buf[cch - 1] != '\\' as _ {
        buf[cch] = '\\' as _;
        cch += 1;
    }

    if buf[0] == '\\' as _ {
        /*\\?\*/
        String::from_utf16_lossy(&buf[4..cch])
    } else {
        String::from_utf16_lossy(&buf[..cch])
    }
}

pub fn alert(msg: &str) {
    let wsz_msg = wsz_from_str(msg);
    let wsz_title = wsz_from_str(APP_TITLE);
    unsafe {
        let hwnd = GetForegroundWindow();
        MessageBoxW(hwnd, pwsz(&wsz_msg), pwsz(&wsz_title), MB_ICONINFORMATION);
    }
}

pub fn get_appdata_local_dir() -> String {
    win32::sh_get_folder_path(None, CSIDL_LOCAL_APPDATA as _, HANDLE::default(), 0)
}

pub fn get_exe_path() -> String {
    win32::get_module_file_name(HMODULE::default()).into_string().unwrap()
}

pub fn file_exists(file_path: &str) -> bool {
    if file_path.is_empty() {
        return false;
    }
    Path::new(file_path).exists()
}

pub fn hiword_usize(value: usize) -> u16 {
    ((value as u32) >> 16) as u16
}

pub fn loword_usize(value: usize) -> u16 {
    (value as u32) as u16
}

pub fn get_x_lparam(lparam: LPARAM) -> i32 {
    loword_usize(lparam.0 as _) as i16 as i32
}

pub fn get_y_lparam(lparam: LPARAM) -> i32 {
    hiword_usize(lparam.0 as _) as i16 as i32
}

pub fn make_lparam(l: i32, h: i32) -> LPARAM {
    LPARAM(((h as u16 as u32) << 16 | l as u16 as u32) as _)
}

pub fn rect_to_u64(rect: RECT) -> u64 {
    let dwtl = (rect.top as u16 as u32) << 16 | rect.left as u16 as u32;
    let dwbr = (rect.bottom as u16 as u32) << 16 | rect.right as u16 as u32;
    (dwtl as u64) << 32 | dwbr as u64
}

pub fn rect_from_u64(value: u64) -> RECT {
    let dwtl = (value >> 32) as u32;
    let dwbr = value as u32;
    let top = (dwtl >> 16) as u16 as i32;
    let left = dwtl as u16 as i32;
    let bottom = (dwbr >> 16) as u16 as i32;
    let right = dwbr as u16 as i32;
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

pub fn iif<T>(condition: bool, true_val: T, false_val: T) -> T {
    if condition {
        true_val
    } else {
        false_val
    }
}
