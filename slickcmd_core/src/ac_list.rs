use crate::app_comm::APP_COMM;
use crate::GLOBAL;
use slickcmd_common::font_info::FontInfo;
use slickcmd_common::winproc::{wndproc, WinProc};
use slickcmd_common::{logd, utils, win32};
use std::cmp;
use std::ffi::c_void;
use std::sync::{LazyLock, Mutex};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Default)]
pub struct AcList {
    pub hwnd: HWND,
    hfont: HFONT,

    item_height: i32,
    delta_per_line: u32,
    items: Vec<String>,
    cur_input: String,

    page_size: i32,

    top: i32,
    sel: i32,

    left_margin: i32,
    down_index: i32,
}

pub static AC_LIST: LazyLock<Mutex<AcList>> = LazyLock::new(|| Mutex::new(AcList::default()));

unsafe impl Send for AcList {}
unsafe impl Sync for AcList {}

impl AcList {
    pub fn exists(&self) -> bool {
        !self.hwnd.is_invalid()
    }

    pub fn create(&mut self, font_info: &FontInfo) -> HWND {
        self.left_margin = 4;
        self.page_size = 16; //

        let window_class = "slck_cmd_ac_list";
        let wsz_class = win32::wsz_from_str(window_class);

        let hinstance = GLOBAL.hinstance();

        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_DROPSHADOW,
            hInstance: hinstance,
            hCursor: win32::load_cursor(IDC_ARROW),
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut c_void),
            lpszClassName: win32::pwsz(&wsz_class),
            lpfnWndProc: Some(wndproc::<Self>),
            ..Default::default()
        };

        let atom = win32::register_class_ex(&wc);
        if atom == 0 {
            logd!("???");
        }

        let style = WS_CHILD | WS_BORDER | WS_CLIPSIBLINGS | WS_OVERLAPPED | WS_VSCROLL;

        let hwnd = win32::create_window_ex(
            WS_EX_TOOLWINDOW,
            window_class,
            "",
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            320,
            win32::get_desktop_window(),
            HMENU::default(),
            hinstance,
            Some(self as *const _ as *const c_void),
        );

        let mut lf = LOGFONTW::default();
        lf.lfWidth = font_info.width;
        lf.lfHeight = font_info.height;
        lf.lfPitchAndFamily = font_info.pitch_and_family;
        lf.lfCharSet = DEFAULT_CHARSET;
        let wsz_facename = win32::wsz_from_str(&font_info.name);
        lf.lfFaceName[..wsz_facename.len()].copy_from_slice(wsz_facename.as_slice());

        self.hfont = win32::create_font_indirect(&lf);

        let hdc = win32::get_dc(hwnd);
        let hfont_old = win32::select_font(hdc, self.hfont);
        let mut tm = TEXTMETRICW::default();
        win32::get_text_metrics(hdc, &mut tm);
        win32::select_font(hdc, hfont_old);
        win32::release_dc(hwnd, hdc);
        self.item_height = tm.tmHeight + 4;

        let mut ul_scrolllines = 0u32;
        win32::system_parameters_info(SPI_GETWHEELSCROLLLINES, 0, &mut ul_scrolllines);
        if ul_scrolllines != 0 {
            self.delta_per_line = WHEEL_DELTA / ul_scrolllines;
        }

        self.hwnd = hwnd;
        hwnd
    }

    pub fn destroy(&mut self) {
        win32::destroy_window(self.hwnd);
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
    }

    pub fn show(&mut self, cur_input: String, x: i32, y: i32, line_height: i32) {
        let y0 = y;
        let y = y + line_height + 4;

        self.cur_input = cur_input;

        let hdc = win32::get_dc(self.hwnd);
        let hfont_old = win32::select_font(hdc, self.hfont);

        let mut max_width = 60;
        let count = self.items.len() as i32;

        let mut sz = SIZE::default();
        for n in 0..count {
            let item = &self.items[n as usize];
            win32::get_text_extend_point(hdc, item, &mut sz);
            if sz.cx > max_width {
                max_width = sz.cx;
            }
        }
        win32::select_font(hdc, hfont_old);
        win32::release_dc(self.hwnd, hdc);

        let cx_sb: i32 = if count > self.page_size {
            win32::get_system_metrics(SM_CXVSCROLL)
        } else {
            0
        };

        let mut si = SCROLLINFO::default();
        si.cbSize = size_of::<SCROLLINFO>() as _;
        si.nMax = count - 1;
        si.nPage = self.page_size as u32;
        si.fMask = SIF_PAGE | SIF_POS | SIF_RANGE;

        win32::set_scroll_info(self.hwnd, SB_VERT, &si, false);
        self.top = 0;
        self.sel = -1;

        let display_count = cmp::min(self.page_size, count);
        let height = self.item_height * display_count + 2;

        let cy_screen = win32::get_system_metrics(SM_CYSCREEN);
        let y = if y + height > cy_screen {
            y0 - height - 3
        } else {
            y
        };

        win32::set_window_pos(
            self.hwnd,
            HWND_TOPMOST,
            x,
            y,
            max_width + cx_sb + self.left_margin * 2 + 3,
            height,
            SWP_SHOWWINDOW,
        );

        win32::invalidate_rect(self.hwnd, None, true);
        win32::update_window(self.hwnd);

        GLOBAL.set_showing_acl(true);
    }

    pub fn is_visible(&self) -> bool {
        !self.hwnd.is_invalid() && win32::is_window_visible(self.hwnd)
    }

    fn on_paint(&self, _wparam: WPARAM, _lparam: LPARAM) {
        let mut ps = PAINTSTRUCT::default();
        win32::begin_paint(self.hwnd, &mut ps);
        let hdc = ps.hdc;

        let mut rect = RECT::default();
        win32::get_client_rect(self.hwnd, &mut rect);

        let hdc_draw = win32::create_compatible_dc(hdc);
        let hbitmap = win32::create_compatible_bitmap(hdc, rect.right, rect.bottom);
        let hbitmap_old = win32::select_object(hdc_draw, hbitmap.into());

        let cx = rect.right;
        win32::fill_rect(hdc_draw, &rect, win32::get_sys_color_brush(COLOR_WINDOW));
        let hfont_old = win32::select_font(hdc_draw, self.hfont);
        win32::set_bk_mode(hdc_draw, TRANSPARENT);

        let mut si = SCROLLINFO::default();
        si.cbSize = size_of::<SCROLLBARINFO>() as _;
        si.fMask = SIF_POS;
        win32::get_scroll_info(self.hwnd, SB_VERT, &mut si);
        let first_item = si.nPos;

        let item_count = self.items.len() as i32;
        let mut count = self.page_size;
        if first_item + count > item_count {
            count = item_count - first_item;
        }
        for n in 0..count {
            self.draw_item(hdc_draw, n, first_item + n, cx);
        }

        win32::bit_blt(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            hdc_draw,
            rect.left,
            rect.top,
            SRCCOPY,
        );

        win32::select_font(hdc_draw, hfont_old);
        win32::select_object(hdc_draw, hbitmap_old);
        win32::delete_dc(hdc_draw);

        win32::end_paint(self.hwnd, &ps);
    }

    fn draw_item(&self, hdc: HDC, draw_index: i32, item_index: i32, width: i32) {
        let mut rect = RECT {
            left: self.left_margin,
            top: draw_index * self.item_height,
            right: width,
            bottom: (draw_index + 1) * self.item_height,
        };
        win32::set_text_color(hdc, win32::get_sys_color(COLOR_WINDOWTEXT));

        let item = &self.items[item_index as usize];
        if item_index == self.sel {
            rect.left = 0;
            win32::fill_rect(hdc, &rect, win32::get_sys_color_brush(COLOR_HIGHLIGHT));
            win32::set_text_color(hdc, win32::get_sys_color(COLOR_HIGHLIGHTTEXT));
            rect.left = self.left_margin;
        }
        let format = DT_LEFT | DT_SINGLELINE | DT_NOPREFIX | DT_VCENTER | DT_END_ELLIPSIS;
        win32::draw_text(hdc, item, &mut rect, format);
    }

    pub fn close(&self) {
        if !win32::is_window_visible(self.hwnd) {
            return;
        }
        win32::show_window(self.hwnd, SW_HIDE);

        APP_COMM.notify_ac_list_closed();

        GLOBAL.set_showing_acl(false);
    }

    fn on_vscroll(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let typ = utils::loword_usize(wparam.0) as i32;

        let mut si = SCROLLINFO::default();
        si.cbSize = size_of::<SCROLLINFO>() as _;
        si.fMask = SIF_ALL;

        win32::get_scroll_info(self.hwnd, SB_VERT, &mut si);
        let mut pos = si.nPos;

        match SCROLLBAR_COMMAND(typ) {
            SB_TOP => {
                pos = si.nMin;
            }
            SB_BOTTOM => {
                pos = si.nMax;
            }
            SB_LINEUP => {
                pos -= 1;
            }
            SB_LINEDOWN => {
                pos += 1;
            }
            SB_PAGEUP => {
                pos -= si.nPage as i32;
            }
            SB_PAGEDOWN => {
                pos += si.nPage as i32;
            }
            SB_THUMBTRACK => {
                pos = si.nTrackPos;
            }
            _ => (),
        };

        si.fMask = SIF_POS;
        let old_pos = si.nPos;
        si.nPos = pos;

        win32::set_scroll_info(self.hwnd, SB_VERT, &si, true);
        win32::get_scroll_info(self.hwnd, SB_VERT, &mut si);
        self.top = si.nPos;

        if old_pos != si.nPos {
            win32::invalidate_rect(self.hwnd, None, true);
            win32::update_window(self.hwnd);
        }
    }

    fn on_mouse_move(&mut self, _wparam: WPARAM, lparam: LPARAM) {
        let y = utils::hiword_usize(lparam.0 as _) as i32;

        let sel = (y / self.item_height) + self.top;
        if sel != self.sel {
            self.sel = sel;
            win32::invalidate_rect(self.hwnd, None, true);
            win32::update_window(self.hwnd);
        }
    }

    fn on_mouse_wheel(&self, wparam: WPARAM, lparam: LPARAM) {
        if self.delta_per_line == 0 {
            return;
        }
        let accum_delta = utils::hiword_usize(wparam.0) as i16;
        for _ in 0..1 {
            let sb = if accum_delta > 0 {
                SB_LINEUP
            } else {
                SB_LINEDOWN
            };
            win32::send_message(self.hwnd, WM_VSCROLL, WPARAM(sb.0 as _), LPARAM(0));
        }
        let mut pt = POINT {
            x: utils::get_x_lparam(lparam),
            y: utils::get_y_lparam(lparam),
        };
        win32::screen_to_client(self.hwnd, &mut pt);
        win32::send_message(
            self.hwnd,
            WM_MOUSEMOVE,
            WPARAM(0),
            utils::make_lparam(pt.x, pt.y),
        );
    }

    fn on_key_down(&mut self, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        let vk = wparam.0 as u16;
        let mut sel = self.sel;
        let max_sel = self.items.len() as i32 - 1;

        if vk == VK_ESCAPE.0 {
            self.on_item_select(-1);
            self.close();
            return LRESULT(0);
        }

        if vk == VK_TAB.0 {
            if sel != -1 {
                self.on_item_select(sel);
                return LRESULT(0);
            }
            return LRESULT(1); //?
        }

        if vk == VK_UP.0 {
            if sel == -1 {
                sel = max_sel;
            } else {
                sel -= 1;
            }
        } else if vk == VK_DOWN.0 {
            if sel == -1 {
                sel = 0;
            } else {
                sel += 1;
                if sel > max_sel {
                    sel = -1;
                }
            }
        } else if vk == VK_PRIOR.0 {
            sel = if sel == -1 {
                max_sel
            } else if sel == 0 {
                -1
            } else {
                cmp::max(0, sel - self.page_size)
            }
        } else if vk == VK_NEXT.0 {
            sel = if sel == -1 {
                0
            } else if sel == max_sel {
                -1
            } else {
                cmp::min(max_sel, sel + self.page_size)
            }
        }
        if self.sel != sel {
            let mut top = self.top;
            if sel == -1 {
                //
            } else if sel < self.top {
                top = sel;
            } else if sel > self.top + self.page_size - 1 {
                top = sel - self.page_size + 1;
            }
            if top != self.top {
                self.top = top;
                win32::set_scroll_pos(self.hwnd, SB_VERT, self.top, true);
            }
            self.sel = sel;
            win32::invalidate_rect(self.hwnd, None, false);
            win32::update_window(self.hwnd);

            //
            self.on_item_select(self.sel);
        }
        LRESULT(0)
    }

    fn on_item_select(&self, index: i32) {
        let mut input: String;
        if index == -1 {
            input = self.cur_input.clone();
        } else {
            input = "cd ".into();
            let cur_input = &self.cur_input;
            if cur_input.len() > 3 && cur_input[3..].trim_start().starts_with("/d ") {
                input.push_str("/d ");
            }
            let item = &self.items[index as usize];
            let has_space = item.contains(' ');
            if has_space {
                input.push('"');
            }
            input.push_str(item);
            if has_space {
                input.push('"');
            }
        }
        APP_COMM.update_input(&input);
    }

    fn on_key_up(&self, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        let vk = wparam.0 as u16;
        if vk == VK_TAB.0 {
            if self.sel != -1 {
                self.close();
                return LRESULT(0);
            }
            return LRESULT(1);
        }
        LRESULT(0)
    }

    fn on_lbutton_down(&mut self, _wparam: WPARAM, _lparam: LPARAM) {
        self.down_index = self.sel;
    }

    fn on_lbutton_up(&self, _wparam: WPARAM, _lparam: LPARAM) {
        if self.down_index == self.sel {
            self.on_item_select(self.sel);
            self.close();
        }
    }
}

impl WinProc for AcList {
    fn wndproc(&mut self, window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_PAINT => {
                self.on_paint(wparam, lparam);
                return LRESULT(0);
            }
            WM_VSCROLL => {
                self.on_vscroll(wparam, lparam);
                return LRESULT(0);
            }
            WM_MOUSEMOVE => {
                self.on_mouse_move(wparam, lparam);
            }
            WM_MOUSEWHEEL => {
                self.on_mouse_wheel(wparam, lparam);
            }
            WM_LBUTTONDOWN => {
                self.on_lbutton_down(wparam, lparam);
            }
            WM_LBUTTONUP => {
                self.on_lbutton_up(wparam, lparam);
            }
            WM_SETTEXT => {
                println!("?");
            }
            WM_KEYDOWN => {
                return self.on_key_down(wparam, lparam);
            }
            WM_KEYUP => {
                return self.on_key_up(wparam, lparam);
            }
            WM_CLOSE => {
                self.close();
                return LRESULT(0);
            }
            WM_NCDESTROY => {
                win32::delete_object(self.hfont.into());
            }
            _ => {}
        }

        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
}
