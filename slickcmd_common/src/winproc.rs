use crate::win32;
use std::ptr;
use windows::Win32::Foundation::{FALSE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn message_loop(haccel: HACCEL) {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0) != FALSE {
            if haccel.is_invalid() || TranslateAcceleratorW(msg.hwnd, haccel, &mut msg) == 0 {
                _ = TranslateMessage(&msg);
                _ = DispatchMessageW(&msg);
            }
        }
    }
}

pub trait WinProc {
    fn wndproc(&mut self, window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

pub extern "system" fn wndproc<T: WinProc>(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let p_self: *mut T;
    if message == WM_NCCREATE {
        let p_cs = win32::lparam_as_ref::<CREATESTRUCTW>(&lparam);
        p_self = p_cs.lpCreateParams as *mut T;
        win32::set_window_long_ptr(hwnd, GWLP_USERDATA, p_self);
    } else {
        p_self = win32::get_window_long_ptr_mut(hwnd, GWLP_USERDATA);
        if p_self == ptr::null_mut() {
            return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
        }
    }
    let r_self = unsafe { &mut *p_self };
    r_self.wndproc(hwnd, message, wparam, lparam)
}
