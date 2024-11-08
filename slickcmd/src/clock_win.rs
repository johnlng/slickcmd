use crate::global::GLOBAL;
use crate::win_man::WinMan;
use crate::wt_focus_man::WtFocusMan;
use slickcmd_common::consts::{WM_SYSTEM_MOVESIZEEND, WM_SYSTEM_MOVESIZESTART, WM_WT_FOCUS_CHANGE};
use slickcmd_common::font_info::FontInfo;
use slickcmd_common::winproc::{wndproc, WinProc};
use slickcmd_common::{logd, win32};
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static WC_REGISTERED: AtomicBool = AtomicBool::new(false);

pub struct ClockWin {
    hwnd: HWND,
    hwnd_term: HWND,

    hfont: HFONT,

    last_console_bounds: RECT,

    width: i32,
    height: i32,

    is_wt: bool,
}

impl ClockWin {
    pub fn new(hwnd_term: HWND) -> ClockWin {
        ClockWin {
            hwnd: HWND::default(),
            hwnd_term,
            hfont: HFONT::default(),
            last_console_bounds: RECT::default(),
            width: 0,
            height: 0,
            is_wt: hwnd_term.0 as usize == WtFocusMan::hwnd_wt(),
        }
    }

    fn register_class(window_class: &str) -> bool {
        let wsz_class = win32::wsz_from_str(window_class);

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hInstance: GLOBAL.hinstance(),
            hCursor: win32::load_cursor(IDC_ARROW),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut c_void),
            lpszClassName: win32::pwsz(&wsz_class),
            lpfnWndProc: Some(wndproc::<Self>),
            ..Default::default()
        };

        let atom = win32::register_class_ex(&wc);
        if atom == 0 {
            return false;
        }
        true
    }

    pub fn create(&mut self, fi: FontInfo, cell_size: (i32, i32)) {
        let window_class = "slck_cmd_clock";
        if !WC_REGISTERED.load(Relaxed) {
            if !Self::register_class(&window_class) {
                debug_assert!(false);
            }
            WC_REGISTERED.store(true, Relaxed);
        }

        let mut lf = LOGFONTW::default();
        lf.lfWidth = fi.width;
        lf.lfHeight = fi.height;
        lf.lfPitchAndFamily = fi.pitch_and_family;
        lf.lfCharSet = DEFAULT_CHARSET;

        let wsz_facename = win32::wsz_from_str(&fi.name);
        lf.lfFaceName[..wsz_facename.len()].copy_from_slice(wsz_facename.as_slice());

        self.hfont = win32::create_font_indirect(&lf);

        //
        self.width = cell_size.0 * "23:45:59".len() as i32;
        self.height = cell_size.1;
        let style = if self.is_wt { WS_POPUP } else { WS_CHILD };
        let hwnd = win32::create_window_ex(
            WINDOW_EX_STYLE::default(),
            &window_class,
            "",
            style,
            0,
            0,
            self.width,
            self.height,
            self.hwnd_term,
            HMENU::default(),
            GLOBAL.hinstance(),
            Some(self as *const _ as *const c_void),
        );
        self.hwnd = hwnd;
        if self.is_wt {
            WtFocusMan::add_wt_listener_hwnd(self.hwnd);
        } else {
            let style = win32::get_window_long(self.hwnd_term, GWL_STYLE);
            let style = WINDOW_STYLE(style as u32);
            if !style.contains(WS_CLIPCHILDREN) {
                let style = style | WS_CLIPCHILDREN;
                win32::set_window_long(self.hwnd_term, GWL_STYLE, style.0 as i32);
            }
        }

        self.on_timer();
        win32::set_timer(self.hwnd, 1, 1000, None);
    }

    pub fn destroy(&mut self) {
        if self.is_wt {
            WtFocusMan::remove_wt_listener_hwnd(self.hwnd);
        }
        if self.hwnd.is_invalid() {
            logd!("??");
            return;
        }
        win32::destroy_window(self.hwnd);
        win32::delete_object(self.hfont.into());
        self.hwnd = HWND::default();
        self.hfont = HFONT::default();
    }

    fn on_timer(&mut self) {
        let console_bounds = WinMan::get_console_bounds(self.hwnd_term);
        let hwnd = self.hwnd;
        if self.last_console_bounds != console_bounds {
            self.last_console_bounds = console_bounds;
            let mut pt = POINT {
                x: console_bounds.right - self.width,
                y: console_bounds.top,
            };
            if self.is_wt {
                win32::client_to_screen(self.hwnd_term, &mut pt);
            }

            win32::move_window(hwnd, pt.x, pt.y, self.width, self.height, false);
            win32::show_window(hwnd, SW_SHOWNOACTIVATE);

            if !self.is_wt {
                win32::invalidate_rect(self.hwnd_term, None, true);
                win32::update_window(self.hwnd_term);
            }
        }

        win32::invalidate_rect(hwnd, None, false);
        win32::update_window(hwnd);
    }

    fn on_paint(&mut self) {
        let mut ps = PAINTSTRUCT::default();
        win32::begin_paint(self.hwnd, &mut ps);
        let hfont_old = win32::select_object(ps.hdc, self.hfont.into());

        let bg_color = win32::rgb(0, 0, 168);
        let hbr = win32::create_solid_brush(bg_color);
        win32::fill_rect(ps.hdc, &ps.rcPaint, hbr);
        win32::delete_object(hbr.into());

        let st = win32::get_local_time();
        let s_time = format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond);
        win32::set_bk_color(ps.hdc, bg_color);
        win32::set_text_color(ps.hdc, win32::rgb(240, 240, 240));

        let mut rc = RECT {
            left: 0,
            top: 0,
            right: self.width,
            bottom: self.height,
        };
        win32::draw_text(
            ps.hdc,
            &s_time,
            &mut rc,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );

        win32::select_object(ps.hdc, hfont_old);
        win32::end_paint(self.hwnd, &ps);
    }
}

impl WinProc for ClockWin {
    fn wndproc(&mut self, window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_TIMER => {
                self.on_timer();
                return LRESULT(0);
            }
            WM_PAINT => {
                self.on_paint();
                return LRESULT(0);
            }
            WM_SYSTEM_MOVESIZESTART => {
                win32::show_window(self.hwnd, SW_HIDE);
            }
            WM_SYSTEM_MOVESIZEEND | WM_WT_FOCUS_CHANGE => {
                self.last_console_bounds = RECT::default();
                self.on_timer();
            }
            _ => {}
        }
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
}
