use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    command_hist::{CommandHist, CommandInfo}
};
use slickcmd_common::font_info::FontInfo;
use slickcmd_common::win32::{self};
use windows::Win32::{
    Foundation::*, Graphics::Gdi::*, UI::Controls::*, UI::Input::KeyboardAndMouse::*,
    UI::WindowsAndMessaging::*,
};
use crate::app::App;
use crate::global::GLOBAL;

#[derive(Default, Clone)]
struct CommandGroup {
    hist: Rc<CommandHist>,
    text: String,
    expanded: bool,
}

enum DisplayItem {
    Group(Rc<RefCell<CommandGroup>>),
    Info(CommandInfo),
}

#[derive(Default)]
pub struct CommandHistList {
    pub hwnd: HWND,

    pub ctrl_id: u16,

    #[allow(dead_code)]
    pub wrap: bool,

    hfont: HFONT,

    pub hists: Vec<Rc<CommandHist>>,
    groups: Vec<Rc<RefCell<CommandGroup>>>,
    display_items: Vec<DisplayItem>,
    collapsed_group_indexes: HashSet<i32>,

    mem_dc: HDC,
    mem_dc_state0: i32,
    client_width: i32,

    htheme: HTHEME,
    group_button_right: i32,
}

const ITEM_PADDING: i32 = 5;

impl CommandHistList {
    pub fn create(&mut self, hwnd_parent: HWND, font_info: &FontInfo) {
        let style = WS_CHILD
            | WS_VISIBLE
            | WS_VSCROLL
            | WS_HSCROLL
            | WINDOW_STYLE((LBS_OWNERDRAWVARIABLE | LBS_NOTIFY) as _);

        let mut rc_parent = RECT::default();
        win32::get_client_rect(hwnd_parent, &mut rc_parent);

        let hinstance = GLOBAL.hinstance();
        let hwnd = win32::create_window_ex(
            WINDOW_EX_STYLE::default(),
            "ListBox",
            "",
            style,
            0,
            0,
            rc_parent.right,
            0,
            hwnd_parent,
            HMENU(self.ctrl_id as _),
            hinstance,
            None,
        );

        self.client_width = rc_parent.right; //

        win32::set_window_subclass(hwnd, Some(s_subclass_proc), 1, self as *const _ as usize);

        self.hwnd = hwnd;

        let mut lf = LOGFONTW::default();
        lf.lfWidth = App::dpi_aware_value(font_info.width);
        lf.lfHeight = App::dpi_aware_value(font_info.height);
        lf.lfPitchAndFamily = font_info.pitch_and_family;
        lf.lfCharSet = DEFAULT_CHARSET;

        let wsz_facename = win32::wsz_from_str(&font_info.name);
        lf.lfFaceName[..wsz_facename.len()].copy_from_slice(wsz_facename.as_slice());
        self.hfont = win32::create_font_indirect(&lf);

        win32::send_message(self.hwnd, WM_SETFONT, WPARAM(self.hfont.0 as _), LPARAM(0));

        let hdc = win32::get_dc(self.hwnd);
        self.mem_dc = win32::create_compatible_dc(hdc);
        win32::release_dc(self.hwnd, hdc);

        self.mem_dc_state0 = win32::save_dc(self.mem_dc);
        win32::select_font(self.mem_dc, self.hfont);

        let count = self.hists.len();
        for n in 0..count {
            let hist = self.hists[n].clone();
            let mut group = self.build_group(hist);
            if self.collapsed_group_indexes.contains(&(n as i32)) {
                group.expanded = true;
            }
            self.groups.push(Rc::new(RefCell::new(group)));
        }

        self.collapsed_group_indexes.clear();

        self.fill_items();

        win32::listbox_setcursel(self.hwnd, self.display_items.len() as i32 - 1);

        self.htheme = win32::open_theme_data(self.hwnd, "TREEVIEW");
    }

    pub fn on_draw_item(&mut self, dis: &DRAWITEMSTRUCT) {
        if dis.itemAction == ODA_FOCUS {
            win32::draw_focus_rect(dis.hDC, &dis.rcItem);
            return;
        }

        let hdc = dis.hDC;
        let rc_item0 = dis.rcItem;
        let item = &self.display_items[dis.itemID as usize];

        let hbr_bg: HBRUSH;
        let clr_bg: COLORREF;
        let clr_fg: COLORREF;

        if let DisplayItem::Group(_) = item {
            hbr_bg = win32::get_sys_color_brush(COLOR_BTNFACE);
            clr_bg = win32::get_sys_color(COLOR_BTNFACE);
            clr_fg = win32::get_sys_color(COLOR_WINDOWTEXT);
        } else if (dis.itemState.0 & ODS_SELECTED.0) != 0 {
            hbr_bg = win32::get_sys_color_brush(COLOR_HIGHLIGHT);
            clr_bg = win32::get_sys_color(COLOR_HIGHLIGHT);
            clr_fg = win32::get_sys_color(COLOR_HIGHLIGHTTEXT);
        } else {
            hbr_bg = win32::get_sys_color_brush(COLOR_WINDOW);
            clr_bg = win32::get_sys_color(COLOR_WINDOW);
            clr_fg = win32::get_sys_color(COLOR_WINDOWTEXT);
        }

        win32::fill_rect(hdc, &rc_item0, hbr_bg);
        win32::set_bk_color(hdc, clr_bg);
        win32::set_text_color(hdc, clr_fg);

        let text = match item {
            DisplayItem::Group(group) => &group.borrow().text,
            DisplayItem::Info(info) => &info.command,
        };

        let mut dt_flags = DT_LEFT | DT_TOP | DT_NOFULLWIDTHCHARBREAK | DT_END_ELLIPSIS;

        let mut rc_item = rc_item0.clone();
        rc_item.top += ITEM_PADDING;
        rc_item.left += ITEM_PADDING;
        rc_item.right -= ITEM_PADDING;

        if let DisplayItem::Group(group) = item {
            let state_id = if group.borrow().expanded {
                GLPS_OPENED
            } else {
                GLPS_CLOSED
            };
            let sz = win32::get_theme_part_size(self.htheme, hdc, TVP_GLYPH.0, state_id.0, TS_TRUE);

            let mut rc_button = RECT {
                left: ITEM_PADDING + 1,
                top: rc_item0.top + (rc_item0.bottom - rc_item0.top - sz.cy) / 2 + 1,
                right: ITEM_PADDING + 1 + sz.cx,
                bottom: 0,
            };
            rc_button.bottom = rc_button.top + sz.cy;
            win32::draw_theme_background(self.htheme, hdc, TVP_GLYPH.0, state_id.0, &rc_button);

            self.group_button_right = rc_button.right;

            rc_item.left += 20;
        } else {
            dt_flags |= DT_WORDBREAK;
        }

        win32::draw_text(hdc, text, &mut rc_item, dt_flags);

        if (dis.itemState.0 & ODS_FOCUS.0) != 0 {
            win32::draw_focus_rect(hdc, &rc_item0);
            return;
        }
    }

    pub fn on_measure_item(&mut self, mis: &mut MEASUREITEMSTRUCT) {
        mis.itemHeight = self.calc_item_height(mis.itemID) as _;
        mis.itemWidth = self.client_width as _;
    }

    pub fn on_reflect_command(&mut self, notify_code: u16) {
        if notify_code == LBN_DBLCLK as _ {
            let sel = win32::listbox_getcursel(self.hwnd);
            if sel != LB_ERR {
                self.on_item_dblclick(sel);
            }
        }
    }

    pub fn get_selected_info(&self) -> CommandInfo {
        let sel = win32::listbox_getcursel(self.hwnd);
        if sel != LB_ERR {
            let item = &self.display_items[sel as usize];
            if let DisplayItem::Info(info) = item {
                return info.clone();
            }
        }
        CommandInfo::new("")
    }

    fn handle_key_right(&mut self) -> bool {
        let sel = win32::listbox_getcursel(self.hwnd);
        if sel == LB_ERR as _ {
            return false;
        }
        let item = &mut self.display_items[sel as usize];
        if let DisplayItem::Group(group) = item {
            if !group.borrow().expanded {
                group.borrow_mut().expanded = true;
                self.fill_items();
                return true;
            }
        }
        false
    }

    fn handle_key_left(&mut self) -> bool {
        let sel = win32::listbox_getcursel(self.hwnd);
        if sel == LB_ERR as _ {
            return false;
        }

        let item = &mut self.display_items[sel as usize];
        match item {
            DisplayItem::Group(group) => {
                if group.borrow().expanded {
                    group.borrow_mut().expanded = false;
                    self.fill_items();
                    return true;
                }
            }
            DisplayItem::Info(_) => {
                for n in (0..=sel).rev() {
                    if let DisplayItem::Group(_) = self.display_items[n as usize] {
                        win32::listbox_setcursel(self.hwnd, n);
                        return true;
                    }
                }
            }
        }

        false
    }

    fn update_item_heights(&mut self) {
        let mut rect = RECT::default();
        win32::get_client_rect(self.hwnd, &mut rect);
        self.client_width = rect.right;
        let count = self.display_items.len();
        for n in 0..count {
            let height = self.calc_item_height(n as _);
            win32::send_message(self.hwnd, LB_SETITEMHEIGHT, WPARAM(n), LPARAM(height as _));
        }
    }

    fn fill_items(&mut self) {
        let top_index = win32::send_message(self.hwnd, LB_GETTOPINDEX, WPARAM(0), LPARAM(0)).0;
        let caret_index = win32::send_message(self.hwnd, LB_GETCARETINDEX, WPARAM(0), LPARAM(0)).0;
        let sel_index = win32::listbox_getcursel(self.hwnd);

        win32::send_message(self.hwnd, WM_SETREDRAW, WPARAM(0), LPARAM(0));
        win32::send_message(self.hwnd, LB_RESETCONTENT, WPARAM(0), LPARAM(0));

        self.display_items.clear();

        let count = self.groups.len();
        for n in 0..count {
            let group = &self.groups[n];

            let di = DisplayItem::Group(group.clone());
            self.display_items.push(di);
            win32::send_message(self.hwnd, LB_ADDSTRING, WPARAM(0), LPARAM(0));

            if group.borrow().expanded {
                for info in &group.borrow().hist.infos {
                    let di = DisplayItem::Info(info.clone());
                    self.display_items.push(di);

                    win32::send_message(self.hwnd, LB_ADDSTRING, WPARAM(0), LPARAM(0));
                }
            }
        }
        self.update_item_heights();
        win32::send_message(self.hwnd, LB_SETTOPINDEX, WPARAM(top_index as _), LPARAM(0));
        win32::send_message(
            self.hwnd,
            LB_SETCARETINDEX,
            WPARAM(caret_index as _),
            LPARAM(0),
        );
        win32::listbox_setcursel(self.hwnd, sel_index);

        win32::send_message(self.hwnd, WM_SETREDRAW, WPARAM(1), LPARAM(0));
    }

    fn on_item_dblclick(&mut self, item_index: i32) {
        let item = &mut self.display_items[item_index as usize];
        if let DisplayItem::Group(group) = item {
            let expanded = group.borrow().expanded;
            group.borrow_mut().expanded = !expanded;
            self.fill_items();
        } else {
            let hwnd_parent = win32::get_parent(self.hwnd);
            win32::post_message(hwnd_parent, WM_COMMAND, WPARAM(IDOK.0 as usize), LPARAM(0));
        }
    }

    fn calc_item_height(&mut self, item_index: u32) -> i32 {
        if self.display_items.is_empty() {
            return 0; //?
        }
        let item = &self.display_items[item_index as usize];
        let text: String;
        match item {
            DisplayItem::Group(_) => {
                text = "(1234567890)".into();
            }
            DisplayItem::Info(info) => {
                text = info.command.clone();
            }
        }

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: self.client_width,
            bottom: 1000,
        };
        rect.right -= ITEM_PADDING * 2;

        let fmt = DT_LEFT | DT_TOP | DT_WORDBREAK | DT_NOFULLWIDTHCHARBREAK | DT_CALCRECT;
        win32::draw_text(self.mem_dc, &text, &mut rect, fmt);

        rect.bottom + ITEM_PADDING * 2
    }

    pub fn extract_time_parts(time: u64) -> (i32, i32, i32, i32, i32) {
        let time = time / 1000;
        let md = time / 1000000 % 10000;
        let m = md / 100;
        let d = md % 100;
        let hms = time % 1000000;
        let h = hms / 10000;
        let ns = hms % 10000;
        let n = ns / 100;
        let s = ns % 100;
        (m as _, d as _, h as _, n as _, s as _)
    }

    fn build_group(&mut self, hist: Rc<CommandHist>) -> CommandGroup {
        let info_count = hist.infos.len();

        let time_from = hist.infos[0].time;
        let (m_from, d_from, h_from, n_from, s_from) = Self::extract_time_parts(time_from);

        let time_to = hist.infos[info_count - 1].time;
        let (m_to, d_to, h_to, n_to, s_to) = Self::extract_time_parts(time_to);

        let mut group = CommandGroup::default();
        group.text = format!(
            "[{:02}-{:02} {:02}:{:02}:{:02}] ~ [{:02}-{:02} {:02}:{:02}:{:02}] {}",
            m_from, d_from, h_from, n_from, s_from, m_to, d_to, h_to, n_to, s_to, info_count
        );
        group.expanded = true;
        group.hist = hist;
        group
    }

    fn subclass_proc(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_LBUTTONUP => {
                let mut pt = POINT::default();
                win32::get_cursor_pos(&mut pt);
                win32::screen_to_client(self.hwnd, &mut pt);
                let sel = win32::listbox_getcursel(self.hwnd);
                if sel != LB_ERR as _ {
                    let item = &mut self.display_items[sel as usize];
                    if let DisplayItem::Group(group) = item {
                        if pt.x < self.group_button_right + 6 {
                            let expanded = group.borrow().expanded;
                            group.borrow_mut().expanded = !expanded;
                            self.fill_items();
                        }
                    }
                }
            }
            WM_KEYDOWN => {
                let vk = wparam.0 as u16;
                if vk == VK_ESCAPE.0 {
                    win32::post_message(
                        win32::get_parent(self.hwnd),
                        WM_COMMAND,
                        WPARAM(IDCANCEL.0 as _),
                        LPARAM(0),
                    );
                } else if vk == VK_LEFT.0 && self.handle_key_left() {
                    return LRESULT(0);
                } else if vk == VK_RIGHT.0 && self.handle_key_right() {
                    return LRESULT(0);
                }
            }
            WM_KEYUP => {
                let vk = wparam.0 as u16;
                if vk == VK_RETURN.0 {
                    let sel = win32::listbox_getcursel(self.hwnd);
                    if sel != LB_ERR as _ {
                        let sel = win32::listbox_getcursel(self.hwnd);
                        if sel != LB_ERR {
                            self.on_item_dblclick(sel);
                        }
                    }
                }
            }
            WM_SIZE => {
                self.update_item_heights();
            }
            WM_NCDESTROY => {
                win32::remove_window_subclass(self.hwnd, Some(s_subclass_proc), 1);
                win32::restore_dc(self.mem_dc, self.mem_dc_state0);
                win32::delete_dc(self.mem_dc);
                win32::delete_object(self.hfont.into());
                win32::close_theme_data(self.htheme);
                self.display_items.clear();
                for (n, group) in self.groups.iter().enumerate() {
                    if !group.borrow().expanded {
                        self.collapsed_group_indexes.insert(n as _);
                    }
                }
                self.groups.clear();
            }
            _ => (),
        }

        win32::def_subclass_proc(hwnd, msg, wparam, lparam)
    }
}

extern "system" fn s_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid_subclass: usize,
    ref_data: usize,
) -> LRESULT {
    let p_self = ref_data as *mut CommandHistList;
    let r_self = unsafe { &mut *p_self };
    r_self.subclass_proc(hwnd, msg, wparam, lparam)
}
