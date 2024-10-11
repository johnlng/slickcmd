use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;

use windows::Win32::Foundation::{HINSTANCE, HWND};

use crate::win32;

#[derive(Default)]
pub struct Global {

    hinstance: AtomicUsize,

    shl_msg: AtomicU32,

    hwnd_msg: AtomicUsize

}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

impl Global {

    pub const fn new() -> Global {
        unsafe { core::mem::zeroed() }
    }

    pub fn init(&self, hinstance: HINSTANCE) -> bool {
        self.hinstance.store(hinstance.0 as usize, Relaxed);
        let shl_msg = win32::register_window_message("SLCK_CMD_SHL_MSG");
        self.shl_msg.store(shl_msg, Relaxed);
        true
    }

    pub fn set_hwnd_msg(&self, hwnd: HWND) {
        self.hwnd_msg.store(hwnd.0 as usize, Relaxed);
    }

    pub fn hwnd_msg(&self) -> HWND {
        let hwnd = self.hwnd_msg.load(Relaxed);
        HWND(hwnd as *mut c_void)
    }

    pub fn shl_msg(&self) -> u32 {
        self.shl_msg.load(Relaxed)
    }

    pub fn hinstance(&self) -> HINSTANCE {
        let hinstance = self.hinstance.load(Relaxed);
        HINSTANCE(hinstance as *mut c_void)
    }
}
