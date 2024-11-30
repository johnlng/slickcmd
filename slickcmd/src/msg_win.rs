use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::size_of;
use std::rc::Rc;

use crate::console::Console;
use crate::console_man::ConsoleMan;
use crate::global::GLOBAL;
use crate::keyboard_input::POST_KEYBOARD_INPUTS;
use crate::win_man::WinMan;
use slickcmd_common::consts::*;
use slickcmd_common::winproc::{wndproc, WinProc};
use slickcmd_common::{logd, utils, win32};
use widestring::U16CStr;
use windows::Win32::{Foundation::*, UI::Input::KeyboardAndMouse::*, UI::WindowsAndMessaging::*};
use crate::tray_wins::{TrayWins};
use crate::wt_focus_man::WtFocusMan;

pub struct MsgWin {
    hwnd: HWND,

    win_man: Rc<RefCell<WinMan>>,

    console_man: Rc<RefCell<ConsoleMan>>,

    shl_msg: u32,

    hhook_mouse_ll: HHOOK,

}

impl MsgWin {
    pub fn new(win_man: Rc<RefCell<WinMan>>, console_man: Rc<RefCell<ConsoleMan>>) -> MsgWin {
        MsgWin {
            hwnd: HWND::default(),
            win_man,
            console_man,
            shl_msg: 0,
            hhook_mouse_ll: HHOOK::default(),
        }
    }

    pub fn create(&mut self) -> HWND {
        let window_class = "slck_cmd_msg";
        let wsz_class = win32::wsz_from_str(window_class);

        let hinstance = GLOBAL.hinstance();

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hInstance: hinstance,
            lpszClassName: win32::pwsz(&wsz_class),
            lpfnWndProc: Some(wndproc::<Self>),
            ..Default::default()
        };

        let atom = win32::register_class_ex(&wc);
        debug_assert!(atom != 0);

        let lparam: *mut c_void;
        lparam = self as *mut _ as *mut c_void;

        let hwnd = win32::create_window_ex(
            WINDOW_EX_STYLE::default(),
            window_class,
            "",
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            HMENU::default(),
            hinstance,
            Some(lparam),
        );

        self.hwnd = hwnd;
        self.shl_msg = win32::register_window_message("SLCK_CMD_SHL_MSG");

        hwnd
    }

    pub fn destroy(&mut self) {
        win32::destroy_window(self.hwnd);
    }

    fn process_shell_activate(&mut self, wparam: WPARAM) -> u32 {
        let hwnd = wparam.0;

        let win_man = self.win_man.borrow_mut();
        if win_man.on_activate(hwnd) {
            1
        } else {
            0
        }
    }

    fn process_wt_term_activate(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let hwnd_fg = win32::get_foreground_window().0 as usize;
        if hwnd_fg != WtFocusMan::hwnd_wt() {
            logd!("BAD FG?");
            return;
        }
        let hwnd_term = wparam.0;
        self.console_man.borrow_mut().on_activate(hwnd_term);
    }

    fn cur_console(&self) -> Option<Rc<RefCell<Console>>> {
        self.win_man.borrow().cur_console().clone()
    }
}

impl WinProc for MsgWin {
    fn wndproc(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if msg == self.shl_msg {
            let ret = self.process_shell_activate(wparam);
            return LRESULT((wparam.0 << 32) as isize | ret as isize);
        }

        match msg {

            WM_TRAY_CALLBACK => {
                let nin_msg = lparam.0 as u16 as u32;
                if nin_msg == NIN_SELECT {
                    let index = utils::hiword_usize(lparam.0 as _) as usize;
                    TrayWins::restore(index);
                }
            }

            WM_NOTIFY_KEY_SUPPRESS_END => {
                if let Some(cur_console) = &self.cur_console() {
                    cur_console.borrow_mut().on_key_suppress_end();
                }
            }

            WM_WT_CONSOLE_ACTIVATE => {
                self.process_wt_term_activate(wparam, lparam);
                return LRESULT(0);
            }

            WM_CLEAN_CONSOLES => {
                self.console_man.borrow_mut().clean();
            }

            WM_POST_CONSOLE_ACTIVATE => {
                let hwnd_console = wparam.0;
                self.console_man.borrow_mut().on_activate(hwnd_console);
            }

            WM_SETTEXT => {
                if wparam.0 == 1 {
                    if let Some(cur_console) = &self.cur_console() {
                        let wsz = unsafe { U16CStr::from_ptr_str(lparam.0 as _) };
                        cur_console.borrow_mut().set_input(&wsz.to_string_lossy());
                    }
                    return LRESULT(0);
                }
                if wparam.0 == 2 {
                    if let Some(cur_console) = &self.cur_console() {
                        let pwsz_command = unsafe { U16CStr::from_ptr_str(lparam.0 as _) };
                        let command = pwsz_command.to_string_lossy();
                        let ctrl_down = win32::get_async_key_state(VK_LCONTROL) < 0;
                        cur_console.borrow_mut().use_command(&command, ctrl_down);
                    }
                }
            }

            WM_SHOW_AUTO_COMPLETE => {
                if let Some(cur_console) = &self.cur_console() {
                    if !cur_console.try_borrow_mut().is_ok() {
                        //?
                        return LRESULT(0);
                    }
                    let mut cur_console = cur_console.borrow_mut();
                    let len = wparam.0;
                    let ptr = lparam.0 as *const String;
                    let items: &[String] = unsafe { std::slice::from_raw_parts(ptr, len) };
                    if items.is_empty() {
                        cur_console.hide_ac_list();
                    } else {
                        let ret = cur_console.show_ac_list(&items);
                        if ret.0 != 2 && !win32::is_debugger_present() {
                            self.hhook_mouse_ll = win32::set_windows_hook_ex(
                                WH_MOUSE_LL,
                                Some(MouseProcLL),
                                HINSTANCE::default(),
                                0,
                            );
                        }
                    }
                }
                return LRESULT(0);
            }

            WM_HIDE_AUTO_COMPLETE => {
                if let Some(cur_console) = &self.cur_console() {
                    cur_console.borrow_mut().hide_ac_list();
                }
                return LRESULT(0);
            }

            WM_MOUSEDOWN_SHOWING_ACL => {
                let pt = POINT {
                    x: utils::get_x_lparam(lparam),
                    y: utils::get_y_lparam(lparam),
                };
                let hwnd_parent = win32::get_desktop_window();
                let class_name = "slck_cmd_ac_list";
                let hwnd_acl =
                    win32::find_window_ex(hwnd_parent, None, Some(class_name), None);
                let mut rect = RECT::default();
                win32::get_window_rect(hwnd_acl, &mut rect);
                if !win32::pt_in_rect(&rect, pt) {
                    if let Some(cur_console) = &mut self.cur_console() {
                        cur_console.borrow_mut().hide_ac_list();
                    }
                }
                return LRESULT(0);
            }

            WM_NOTIFY_AC_LIST_CLOSED => {
                win32::unhook_widows_hook_ex(self.hhook_mouse_ll);
                self.hhook_mouse_ll = HHOOK::default();
            }

            WM_HIST_WIN_DESTROYED => {
                if let Some(console) = self.console_man.borrow().get_console(wparam.0) {
                    console.borrow_mut().notify_hist_win_destroyed();
                }
            }

            WM_POST_ACTION => {
                if wparam.0 == 0 {
                    if let Some(mut post_ki) = POST_KEYBOARD_INPUTS.fetch() {
                        if post_ki.hwnd_target == win32::get_foreground_window() {
                            post_ki.keyboard_input.send(lparam.0 == 1);
                        }
                    }
                } else if lparam.0 == win32::get_foreground_window().0 as _ {
                    if let Some(cur_console) = &self.cur_console() {
                        cur_console.borrow_mut().handle_post_action(wparam.0);
                    }
                }
            }

            WM_SHOW_MENU_RESULT => {
                let cmd = wparam.0 as i32;
                if let Some(cur_console) = &self.cur_console() {
                    cur_console.borrow_mut().handle_show_menu_result(cmd);
                }
                return LRESULT(0);
            }

            WM_CORE_KEYDOWN => {
                if let Some(cur_console) = &self.cur_console() {
                    let vk = VIRTUAL_KEY(wparam.0 as u16);
                    let alt_down = lparam.0 != 0;
                    match cur_console.try_borrow_mut() {
                        Ok(mut console) => {
                            if console.handle_key_down(vk, alt_down) {
                                return LRESULT(1);
                            }
                        }
                        Err(_) => {
                            logd!("failed to borrow mut console on keydown.");
                        }
                    }
                }
            }
            WM_CORE_KEYUP => {
                // logd!("@core key up: {}", wparam.0);
                if let Some(cur_console) = &mut self.cur_console() {
                    let vk = VIRTUAL_KEY(wparam.0 as u16);
                    let alt_down = lparam.0 != 0;
                    match cur_console.try_borrow_mut() {
                        Ok(mut console) => {
                            if console.handle_key_up(vk, alt_down) {
                                return LRESULT(1);
                            }
                        }
                        Err(_) => {
                            logd!("failed to borrow mut console on keyup.");
                        }
                    }
                }
            }

            WM_HOTKEY => {
                let id = wparam.0 as i32;
                if id == 1 {
                    if let Some(cur_console) = &mut self.cur_console() {
                        cur_console.borrow().clear();
                    }
                    return LRESULT(0);
                }
            }

            WM_DESTROY => {
                TrayWins::restore_all();
            }

            _ => (),
        }

        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
}

#[allow(non_snake_case)]
extern "system" fn MouseProcLL(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }
    let llhs = &unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };
    let mouse_msg = wparam.0 as u32;
    if mouse_msg == WM_LBUTTONDOWN
        || mouse_msg == WM_RBUTTONDOWN
        || mouse_msg == WM_NCLBUTTONDOWN
        || mouse_msg == WM_NCRBUTTONDOWN
    {
        let lparam_xy = utils::make_lparam(llhs.pt.x, llhs.pt.y);
        return win32::send_message(
            GLOBAL.hwnd_msg(),
            WM_MOUSEDOWN_SHOWING_ACL,
            WPARAM(0),
            lparam_xy,
        );
    }
    LRESULT(0)
}
