use std::ffi::c_void;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicUsize};

use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::UI::WindowsAndMessaging::HHOOK;

#[derive(Default)]
pub struct Global {

    hinstance: AtomicUsize,

    hwnd_target: AtomicUsize,

    hhook: AtomicUsize,

    suppress_hook: AtomicBool,

    showing_acl: AtomicBool,

    showing_menu: AtomicBool,

}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

impl Global {

    pub const fn new() -> Global {
        unsafe { core::mem::zeroed() }
    }

    pub fn init(&self, hinstance: HINSTANCE) -> bool {
        self.hinstance.store(hinstance.0 as usize, Relaxed);
        true
    }

    pub fn hinstance(&self) -> HINSTANCE {
        let hinstance = self.hinstance.load(Relaxed);
        HINSTANCE(hinstance as *mut c_void)
    }

    pub fn hwnd_target(&self) -> HWND {
        let hwnd = self.hwnd_target.load(Relaxed);
        HWND(hwnd as *mut c_void)
    }

    pub fn set_hwnd_target(&self, value: HWND) {
        self.hwnd_target.store(value.0 as usize, Relaxed);
    }

    pub fn hhook(&self) -> HHOOK {
        let hhook = self.hhook.load(Relaxed);
        HHOOK(hhook as *mut c_void)
    }

    pub fn set_hhook(&self, value: HHOOK) {
        self.hhook.store(value.0 as usize, Relaxed);
    }

    pub fn suppress_hook(&self) -> bool {
        self.suppress_hook.load(Relaxed)
    }

    pub fn set_suppress_hook(&self, value: bool) {
        self.suppress_hook.store(value, Relaxed);
    }

    pub fn showing_acl(&self) -> bool {
        self.showing_acl.load(Relaxed)
    }

    pub fn set_showing_acl(&self, value: bool) {
        self.showing_acl.store(value, Relaxed);
    }

    pub fn showing_menu(&self) -> bool {
        self.showing_menu.load(Relaxed)
    }

    pub fn set_showing_menu(&self, value: bool) {
        self.showing_menu.store(value, Relaxed);
    }

}
