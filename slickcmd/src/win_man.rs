use crate::console::Console;
use crate::console_man::ConsoleMan;
use crate::global::GLOBAL;
use crate::wt_focus_man::WtFocusMan;
use slickcmd_common::consts::{WM_POST_CONSOLE_ACTIVATE};
use slickcmd_common::{logd, win32};
use std::cell::{RefCell};
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct WinMan {

    console_man: Rc<RefCell<ConsoleMan>>,

    last_active_hwnd: AtomicUsize,
}

impl WinMan {
    pub fn new(console_man: Rc<RefCell<ConsoleMan>>) -> WinMan {
        WinMan {
            console_man,
            last_active_hwnd: AtomicUsize::new(0),
        }
    }

    fn last_active_hwnd(&self) -> usize {
        self.last_active_hwnd.load(Ordering::Relaxed)
    }

    fn set_last_active_hwnd(&self, hwnd: usize) {
        self.last_active_hwnd.store(hwnd, Ordering::Relaxed);
    }

    pub fn cur_console(&self) -> Option<Rc<RefCell<Console>>> {
        self.console_man.borrow_mut().cur_console()
    }

    fn _on_deactivate(&self, hwnd: usize) {
        // logd!("@ on deactivate.. {}", hwnd);

        if !win32::get_parent(HWND(hwnd as _)).is_invalid() {
            return;
        }

        if hwnd == WtFocusMan::hwnd_wt() {
            WtFocusMan::on_wt_deactivate();
        }

        let hwnd_core_msg = win32::find_window_ex(
            HWND_MESSAGE,
            None,
            Some("slck_cmd_core_msg"),
            None,
        );

        if hwnd_core_msg.is_invalid() {
            logd!("core msg win not found?");
        } else {
            win32::send_message(hwnd_core_msg, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    pub fn on_activate(&self, hwnd: usize) -> bool {
        // logd!("on activate: {}", hwnd);

        let last_active_hwnd = self.last_active_hwnd();
        if hwnd == last_active_hwnd {
            return false;
        }

        if last_active_hwnd != 0 {
            self._on_deactivate(last_active_hwnd);
        }

        let hwnd_ = HWND(hwnd as *mut c_void);
        let class_name = win32::get_class_name(hwnd_);

        if class_name != "ConsoleWindowClass" && class_name != "PseudoConsoleWindow" {
            if class_name == "CASCADIA_HOSTING_WINDOW_CLASS" {
                WtFocusMan::on_wt_activate(hwnd);
                self.set_last_active_hwnd(hwnd);
                return true;
            }
            let borrow = self.console_man.try_borrow_mut();
            if borrow.is_err() { //?
                return false;
            }
            borrow.unwrap().on_activate(0);
            self.set_last_active_hwnd(0);
            return false;
        }

        win32::post_message(
            GLOBAL.hwnd_msg(),
            WM_POST_CONSOLE_ACTIVATE,
            WPARAM(hwnd),
            LPARAM(0),
        );
        self.set_last_active_hwnd(hwnd);
        true
    }

    pub fn get_console_bounds(hwnd_term: HWND) -> RECT { //client coordinates in term window
        let mut rect = RECT::default();
        if hwnd_term.0 as usize == WtFocusMan::hwnd_wt() {
            rect = WtFocusMan::get_console_bounds();
        }
        else {
            win32::get_client_rect(hwnd_term, &mut rect);
        }
        rect
    }
}
