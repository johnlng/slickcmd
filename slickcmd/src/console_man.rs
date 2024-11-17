use crate::app_state::AppState;
use crate::console::Console;
use crate::global::GLOBAL;
use slickcmd_common::consts::WM_CLEAN_CONSOLES;
use slickcmd_common::{logd, win32};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use windows::Win32::Foundation::*;

pub struct ConsoleMan {
    app_state: Rc<AppState>,

    console_map: HashMap<usize, Rc<RefCell<Console>>>,
    cur_console: Option<Rc<RefCell<Console>>>,
}

impl ConsoleMan {
    pub fn new(app_state: Rc<AppState>) -> ConsoleMan {
        ConsoleMan {
            app_state,
            console_map: HashMap::new(),
            cur_console: None,
        }
    }

    pub fn get_console(&self, hwnd: usize) -> Option<Rc<RefCell<Console>>> {
        self.console_map.get(&hwnd).cloned()
    }

    pub fn clean(&mut self) {
        let cur_hwnd = if let Some(console) = &self.cur_console {
            console.borrow().hwnd.0 as usize
        } else {
            0
        };
        let mut invalid_hwnds: Vec<usize> = Vec::new();
        for &hwnd in self.console_map.keys() {
            if !win32::is_window(HWND(hwnd as _)) {
                invalid_hwnds.push(hwnd);
            }
        }
        for hwnd in invalid_hwnds {
            if hwnd == cur_hwnd {
                logd!("something wrong..");
                self.cur_console = None;
            }
            let console = self.console_map.get(&hwnd).unwrap();
            console.borrow_mut().dispose();
            self.console_map.remove(&hwnd);
        }
    }

    fn _on_deactivate(&mut self, hwnd: usize) {
        let console = self.get_console(hwnd);
        if console.is_none() {
            return; //?
        }
        let console_borrow = console.unwrap();
        let mut console = console_borrow.borrow_mut();
        console.on_deactivate();
    }

    pub fn on_activate(&mut self, hwnd: usize) {
        let hwnd = if win32::is_window(HWND(hwnd as _)) {
            hwnd
        } else {
            0
        };

        win32::post_message(GLOBAL.hwnd_msg(), WM_CLEAN_CONSOLES, WPARAM(0), LPARAM(0));
        if let Some(console) = &self.cur_console {
            let cur_hwnd = console.borrow().hwnd.0 as usize;
            if cur_hwnd == hwnd {
                console.borrow_mut().on_activate();
                return;
            } else {
                self._on_deactivate(cur_hwnd);
            }
        }
        if hwnd == 0 {
            return;
        }
        let hwnd_ = HWND(hwnd as _);
        let console = self
            .console_map
            .entry(hwnd)
            .or_insert_with(|| Rc::new(RefCell::new(Console::new(hwnd_, self.app_state.clone()))));

        console.borrow_mut().on_activate();
        self.cur_console = Some(console.clone());
    }

    pub fn contains(&self, hwnd: usize) -> bool {
        self.console_map.contains_key(&hwnd)
    }

    pub fn cur_console(&mut self) -> Option<Rc<RefCell<Console>>> {
        let mut invalid = false;
        if let Some(console) = &self.cur_console {
            match console.try_borrow() {
                Err(_) => return None,
                Ok(console) => {
                    if !console.check_valid() {
                        invalid = true;
                    }
                }
            }
        }
        if invalid {
            self.cur_console = None;
        }
        self.cur_console.clone()
    }
}
