use crate::win32;
use crate::win32::wsz_from_str;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering::Relaxed;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, HWND_MESSAGE, WM_SETTEXT};

static HWND_MSG_VAL: AtomicIsize = AtomicIsize::new(0);

pub fn log(msg: &str) {
    let mut hwnd_msg_val = HWND_MSG_VAL.load(Relaxed);
    if hwnd_msg_val == 0 {
        let hwnd = win32::find_window_ex(HWND_MESSAGE, None, Some("slck_cmd_log"), None);
        hwnd_msg_val = if hwnd.is_invalid() { -1 } else { hwnd.0 as _ };
        HWND_MSG_VAL.store(hwnd_msg_val, Relaxed);
    }
    if hwnd_msg_val == -1 || hwnd_msg_val == 0 || msg.is_empty() {
        return;
    }
    let hwnd_msg = HWND(hwnd_msg_val as _);
    let wsz = wsz_from_str(msg);
    unsafe {
        SendMessageW(
            hwnd_msg,
            WM_SETTEXT,
            WPARAM(0),
            LPARAM(wsz.as_ptr() as isize),
        );
    }
}

pub fn cls() {
    log("::CLS");
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        $crate::logger::log(&res);
    }}
}

#[macro_export]
macro_rules! logd {
    ($($arg:tt)*) => {{
        if cfg!(debug_assertions) {
            let res = std::fmt::format(format_args!($($arg)*));
            $crate::logger::log(&res);
        }
    }}
}
