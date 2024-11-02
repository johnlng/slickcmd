use std::cmp::max;
use std::ffi::c_void;
use std::mem::size_of;
use std::rc::Rc;

use slickcmd_common::{consts::*, win32};
use windows::Win32::{
    Foundation::*, Graphics::Gdi::*, UI::Controls::*, UI::WindowsAndMessaging::*,
};

use crate::command_hist::CommandHist;
use crate::command_hist_list::CommandHistList;
use crate::global::GLOBAL;
use slickcmd_common::font_info::FontInfo;
use slickcmd_common::winproc::{wndproc, WinProc};

// #[derive(Default)]
pub struct CommandHistWin {
    pub hwnd: HWND,

    pub hists: Vec<Rc<CommandHist>>,

    pub font_info: FontInfo,

    hwnd_console: HWND,

    list: CommandHistList,
}

impl CommandHistWin {
    pub fn new(hwnd_console: HWND) -> CommandHistWin {
        CommandHistWin {
            hwnd: HWND::default(),
            hists: Vec::new(),
            font_info: FontInfo::default(),
            hwnd_console,
            list: CommandHistList::default(),
        }
    }

    pub fn create(&mut self, hwnd_owner: HWND) -> HWND {
        let window_class = "slck_cmd_command_hist";
        let wsz_class = win32::wsz_from_str(window_class);

        let hinstance = GLOBAL.hinstance();

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc::<Self>),
            hInstance: hinstance,
            hIcon: win32::load_icon(hinstance, IDI_SLICKCMD),
            hCursor: win32::load_cursor(IDC_ARROW),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut c_void),
            lpszClassName: win32::pwsz(&wsz_class),
            hIconSm: win32::load_icon(hinstance, IDI_SMALL),

            ..Default::default()
        };

        let atom = win32::register_class_ex(&wc);
        if atom == 0 && win32::get_last_error() != ERROR_CLASS_ALREADY_EXISTS {
            debug_assert!(false);
        }

        let mut rect = RECT::default();
        win32::get_window_rect(hwnd_owner, &mut rect);

        let owner_w = rect.right - rect.left;
        let owner_h = rect.bottom - rect.top;
        let w = max(400, (owner_w * 2 / 3) as i32);
        let h = max(400, (owner_h * 2 / 3) as i32);

        let x = rect.left + (owner_w - w) / 2;
        let y = rect.top + (owner_h - h) / 2;

        let hwnd = win32::create_window_ex(
            WS_EX_TOOLWINDOW,
            window_class,
            "Command History",
            WS_POPUPWINDOW | WS_CAPTION | WS_SIZEBOX | WS_MAXIMIZEBOX | WS_VISIBLE,
            x,
            y,
            w,
            h,
            hwnd_owner,
            HMENU::default(),
            hinstance,
            Some(self as *const _ as *const c_void),
        );

        self.hwnd = hwnd;

        //
        self.list.ctrl_id = 100;
        self.list.hists = self.hists.clone();
        self.list.create(self.hwnd, &self.font_info);

        self.layout();
        win32::set_focus(self.list.hwnd);

        win32::set_timer(self.hwnd, 1, 200, None);

        self.hwnd
    }

    fn layout(&mut self) {
        let mut rect = RECT::default();
        win32::get_client_rect(self.hwnd, &mut rect);
        let w = rect.right;
        let h = rect.bottom;
        if w <= 0 {
            return; //?
        }
        win32::move_window(self.list.hwnd, 0, 0, w, h, true);
    }

    pub fn exists(&self) -> bool {
        !self.hwnd.is_invalid()
    }

    pub fn destroy(&mut self) {
        win32::destroy_window(self.hwnd);
    }
}

impl WinProc for CommandHistWin {
    fn wndproc(&mut self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                win32::begin_paint(hwnd, &mut ps);
                win32::end_paint(self.hwnd, &ps);
            }
            WM_SIZE => {
                self.layout();
            }
            WM_TIMER => {
                if wparam.0 == 1 {
                    if !win32::is_window(win32::get_parent(self.hwnd)) {
                        win32::send_message(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
            }
            WM_COMMAND => {
                let id = wparam.0 as u16;
                if id == IDCANCEL.0 as _ {
                    win32::send_message(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                } else if id == IDOK.0 as _ {
                    let info = self.list.get_selected_info();
                    if info.command.is_empty() {
                        return LRESULT(0);
                    }
                    let wsz_command = win32::wsz_from_str(&info.command);
                    win32::send_message(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                    win32::send_message(
                        GLOBAL.hwnd_msg(),
                        WM_SETTEXT,
                        WPARAM(2),
                        LPARAM(wsz_command.as_ptr() as _),
                    );
                } else if id == self.list.ctrl_id {
                    self.list.on_reflect_command((wparam.0 as u32 >> 16) as u16);
                    return LRESULT(0);
                }
            }
            WM_DRAWITEM => {
                if wparam.0 as u16 == self.list.ctrl_id {
                    let pdis = lparam.0 as *const c_void as *const DRAWITEMSTRUCT;
                    self.list.on_draw_item(&unsafe { *pdis });
                    return LRESULT(1);
                }
            }
            WM_MEASUREITEM => {
                if wparam.0 as u16 == self.list.ctrl_id {
                    let pmis = lparam.0 as *mut c_void as *mut MEASUREITEMSTRUCT;
                    self.list.on_measure_item(&mut unsafe { *pmis })
                }
            }
            WM_NCDESTROY => {
                self.hists.clear();

                let hwnd_msg = GLOBAL.hwnd_msg();
                // let hwnd_console = win32::get_parent(self.hwnd);
                let hwnd_console = self.hwnd_console;
                if win32::is_window(hwnd_console) {
                    let wparam = WPARAM(hwnd_console.0 as _);
                    win32::send_message(hwnd_msg, WM_HIST_WIN_DESTROYED, wparam, LPARAM(0));
                }

                self.hwnd = HWND::default();
            }
            _ => (),
        }
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}
