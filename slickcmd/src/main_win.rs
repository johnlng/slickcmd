use crate::options_dlg::OptionsDlg;
use slickcmd_common::winproc::{wndproc, WinProc};
use slickcmd_common::{consts::*, win32};
use std::env;
use std::ffi::c_void;
use std::mem::size_of;
use windows::Win32::UI::Controls::{NMHDR, NM_CLICK};
use windows::{
    core::PCWSTR,
    Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};
use crate::global::GLOBAL;

#[derive(Default)]
pub struct MainWin {
    pub hwnd: HWND,
}

impl MainWin {
    pub fn create(&mut self) -> HWND {
        let window_class = "slck_cmd_main";
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
            lpszMenuName: PCWSTR(IDC_SLICKCMD as *const u16),
            lpszClassName: win32::pwsz(&wsz_class),
            hIconSm: win32::load_icon(hinstance, IDI_SMALL),

            ..Default::default()
        };

        let atom = win32::register_class_ex(&wc);
        debug_assert!(atom != 0);

        let mut rc_workarea = RECT::default();
        win32::system_parameters_info(SPI_GETWORKAREA, 0, &mut rc_workarea);

        let mut rc_win = RECT {
            left: 0,
            top: 0,
            right: 400,
            bottom: 400,
        };
        win32::adjust_window_rect(&mut rc_win, WS_OVERLAPPEDWINDOW, false);

        let w = rc_win.right - rc_win.left;
        let h = rc_win.bottom - rc_win.top;
        let x = rc_workarea.left + (rc_workarea.right - rc_workarea.left - w) / 2;
        let y = rc_workarea.top + (rc_workarea.bottom - rc_workarea.top - h) / 2;

        let hwnd = win32::create_window_ex(
            WINDOW_EX_STYLE::default(),
            window_class,
            APP_TITLE,
            WS_OVERLAPPEDWINDOW,
            x,
            y,
            w,
            h,
            HWND::default(),
            HMENU::default(),
            hinstance,
            Some(self as *const _ as *const c_void),
        );

        self.hwnd = hwnd;
        hwnd
    }

    fn process_tray_callback(&mut self, wparam: WPARAM, lparam: LPARAM) {
        let nin_msg = lparam.0 as u32;

        match nin_msg {
            NIN_KEYSELECT | NIN_SELECT | WM_LBUTTONUP => {
                // win32::show_window(self.hwnd, SW_SHOW);
            }
            WM_RBUTTONUP => {
                let x = wparam.0 as u16 as i16 as i32;
                let y = (wparam.0 >> 16) as u16 as i16 as i32;
                self.show_tray_menu(x, y);
            }
            _ => (),
        }
    }

    fn show_tray_menu(&mut self, x: i32, y: i32) {
        let hmenu = win32::create_popup_menu();
        win32::append_menu(hmenu, MF_STRING, IDM_OPTIONS, Some("&Show Options"));
        win32::append_menu(hmenu, MF_SEPARATOR, 0, Some("-"));

        let hmenu_help = win32::create_popup_menu();
        win32::append_menu(hmenu_help, MF_STRING, IDM_MANUAL, Some("&Manual"));
        win32::append_menu(hmenu_help, MF_STRING, IDM_ABOUT, Some("&About..."));
        win32::append_sub_menu(hmenu, hmenu_help, "&Help");

        win32::append_menu(hmenu, MF_SEPARATOR, 0, Some("-"));
        win32::append_menu(hmenu, MF_STRING, IDM_EXIT, Some("&Exit"));
        win32::set_foreground_window(self.hwnd);
        if win32::track_popup_menu(hmenu, TPM_RIGHTBUTTON, x, y, self.hwnd) == 0 {
            println!("?");
        }
        win32::destroy_menu(hmenu);
    }
}

impl WinProc for MainWin {
    fn wndproc(&mut self, window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_TRAY_CALLBACK => {
                self.process_tray_callback(wparam, lparam);
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xffff) as u16;
                if id == IDM_OPTIONS {
                    let mut dlg = OptionsDlg::new();
                    dlg.show();
                } else if id == IDM_ABOUT {
                    win32::dialog_box(
                        GLOBAL.hinstance(),
                        IDD_ABOUTBOX,
                        self.hwnd,
                        Some(about_dlg_proc),
                    );
                } else if id == IDM_MANUAL {
                    let exe_path = env::current_exe().unwrap().to_string_lossy().to_string();
                    win32::shell_execute(
                        self.hwnd,
                        "open",
                        &exe_path,
                        Some("--man"),
                        None,
                        SW_NORMAL,
                    );
                } else if id == IDM_EXIT {
                    win32::destroy_window(self.hwnd);
                }
                LRESULT(0)
            }
            WM_PAINT => {
                unsafe {
                    _ = ValidateRect(window, None);
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                win32::show_window(self.hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_DESTROY => {
                win32::post_quit_message(0);
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        }
    }
}

extern "system" fn about_dlg_proc(
    h_dlg: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    match message {
        WM_INITDIALOG => 1,
        WM_COMMAND => {
            let id = (wparam.0 as i32) & 0xffff;
            if id == IDOK.0 || id == IDCANCEL.0 {
                win32::end_dialog(h_dlg, MESSAGEBOX_RESULT(id as _))
            }
            1
        }
        WM_NOTIFY => {
            let pnmhdr = lparam.0 as *const NMHDR;
            let nmhdr = &unsafe { *pnmhdr };
            if nmhdr.idFrom == IDC_SYSLINK_SITE as _ && nmhdr.code == NM_CLICK {
                let url = "https://github.com/johnlng/slickcmd";
                win32::shell_execute(h_dlg, "Open", url, None, None, SW_SHOWDEFAULT);
            }
            0
        }
        _ => 0,
    }
}
