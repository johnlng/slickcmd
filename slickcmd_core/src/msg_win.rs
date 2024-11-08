use crate::ac_list::AC_LIST;
use crate::{CORE, GLOBAL};
use slickcmd_common::consts::*;
use slickcmd_common::font_info::FontInfo;
use slickcmd_common::win32;
use slickcmd_common::winproc::{wndproc, WinProc};
use std::ffi::c_void;
use std::mem::size_of;
use widestring::U16CString;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct MsgWin {
    pub hwnd: HWND,
}

impl MsgWin {
    pub fn create(&mut self) -> HWND {
        let hinstance = GLOBAL.hinstance();

        let window_class = "slck_cmd_core_msg";
        let wsz_class = win32::wsz_from_str(window_class);

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hInstance: hinstance,
            lpszClassName: win32::pwsz(&wsz_class),
            lpfnWndProc: Some(wndproc::<Self>),
            ..Default::default()
        };

        let atom = win32::register_class_ex(&wc);
        if atom == 0 && win32::get_last_error() != ERROR_CLASS_ALREADY_EXISTS {
            debug_assert!(false);
        }

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
        hwnd
    }

    fn show_menu(&mut self, wparam: WPARAM, lparam: LPARAM) {
        GLOBAL.set_showing_menu(true);
        let x = lparam.0 as i32 >> 16;
        let y = lparam.0 as u16 as i32;
        let hmenu: HMENU = HMENU(wparam.0 as *mut c_void);
        let cmd = win32::track_popup_menu(
            hmenu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
            x,
            y,
            GLOBAL.hwnd_target(),
        );

        win32::destroy_menu(hmenu);

        let hwnd_app_msg =
            win32::find_window_ex(HWND_MESSAGE, None, Some("slck_cmd_msg"), None);

        win32::post_message(
            hwnd_app_msg,
            WM_SHOW_MENU_RESULT,
            WPARAM(cmd as _),
            LPARAM(0),
        );
        GLOBAL.set_showing_menu(false);
    }

    fn show_ac_list(&self, data: &str) -> LRESULT {
        let lines = data.split('\n');

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut h: i32 = 0;
        let mut items = Vec::<String>::new();
        let mut font_info = FontInfo::default();
        let mut cur_input = String::new();

        for (n, line) in lines.enumerate() {
            if line.is_empty() {
                break;
            }
            match n {
                0 => x = line.parse().unwrap_or_default(),
                1 => y = line.parse().unwrap_or_default(),
                2 => h = line.parse().unwrap_or_default(),
                3 => font_info.name = line.into(),
                4 => font_info.height = line.parse().unwrap_or_default(),
                5 => font_info.width = line.parse().unwrap_or_default(),
                6 => font_info.pitch_and_family = line.parse().unwrap_or_default(),
                7 => cur_input = line.into(),
                _ => items.push(line.into()),
            }
        }

        let mut acl = AC_LIST.lock().unwrap();
        let result = if !acl.exists() {
            acl.create(&font_info);
            LRESULT(0)
        } else if acl.is_visible() {
            LRESULT(2)
        } else {
            LRESULT(1)
        };

        acl.set_items(items);
        acl.show(cur_input, x, y, h);

        result
    }

    fn hide_ac_list(&self) {
        let acl = AC_LIST.lock().unwrap();
        acl.close();
    }
}

impl WinProc for MsgWin {
    fn wndproc(&mut self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_SHOW_MENU => {
                self.show_menu(wparam, lparam);
            }
            WM_SETTEXT => {
                if wparam.0 == 1 {
                    return LRESULT(0);
                } else if wparam.0 == 2 {
                    let data = unsafe { U16CString::from_ptr_str(lparam.0 as *const u16) };
                    return self.show_ac_list(&data.to_string_lossy());
                } else if wparam.0 == 3 {
                    self.hide_ac_list();
                    return LRESULT(0);
                }
            }
            WM_CORE_SUPPRESS_INPUT_EVENT => {
                // let suppress_hook_count = wparam.0 as u32;
                // if suppress_hook_count != 0 {
                    // GLOBAL.set_suppress_input_event_count(GLOBAL.suppress_input_event_count() + suppress_hook_count);
                // }
                // else {
                    GLOBAL.set_suppress_input_event(true);
                // }
                return LRESULT(0);
            }
            WM_CLOSE => {
                win32::destroy_window(hwnd);
            }
            WM_DESTROY => {
                CORE.lock().unwrap().detach();
            }
            _ => (),
        }
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}
