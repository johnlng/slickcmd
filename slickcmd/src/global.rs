use std::cell::{Cell};

use slickcmd_common::win32;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use crate::options::Options;

#[derive(Default)]
pub struct Global {

    hinstance: Cell<HINSTANCE>,
    hwnd_main: Cell<HWND>,
    hwnd_msg: Cell<HWND>,

    pub options: Options,
}

unsafe impl Send for Global{}
unsafe impl Sync for Global{}

impl Global {
    
    pub const fn new() -> Global {
        unsafe { core::mem::zeroed() }
    }

    pub fn init(&self) -> bool {
        let hmodule = win32::get_module_handle();
        self.hinstance.set(HINSTANCE(hmodule.0));
        self.options.init();
        true
    }

    pub fn hinstance(&self) -> HINSTANCE {
        self.hinstance.get()
    }

    pub fn hwnd_main(&self) -> HWND {
        self.hwnd_main.get()
    }

    pub fn set_hwnd_main(&self, hwnd_main: HWND) {
        self.hwnd_main.set(hwnd_main);
    }

    pub fn hwnd_msg(&self) -> HWND {
        self.hwnd_msg.get()
    }

    pub fn set_hwnd_msg(&self, hwnd_msg: HWND) {
        self.hwnd_msg.set(hwnd_msg);
    }

}
