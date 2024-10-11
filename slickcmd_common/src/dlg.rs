use crate::win32;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static CREATING_DLG_ADDR: AtomicUsize = AtomicUsize::new(0);

pub trait Dlg {
    fn set_hwnd(&mut self, hwnd: HWND);
    fn dlg_proc(&mut self, window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> isize;
}

pub fn set_creating_dlg<T: Dlg>(dlg: &T) {
    CREATING_DLG_ADDR.store(dlg as *const T as usize, Relaxed);
}

pub extern "system" fn dlg_proc<T: Dlg>(
    h_dlg: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    let mut p_dlg: *mut T = win32::get_window_long_ptr_mut(h_dlg, GWLP_USERDATA);
    let r_dlg: &mut T;
    if p_dlg.is_null() {
        p_dlg = CREATING_DLG_ADDR.load(Relaxed) as _;
        win32::set_window_long_ptr(h_dlg, GWLP_USERDATA, p_dlg);
        r_dlg = unsafe { &mut *p_dlg };
        r_dlg.set_hwnd(h_dlg);
    }
    else {
        r_dlg = unsafe { &mut *p_dlg };
    }
    r_dlg.dlg_proc(h_dlg, message, wparam, lparam)
}
