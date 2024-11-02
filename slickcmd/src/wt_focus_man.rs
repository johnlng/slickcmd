use crate::global::GLOBAL;
use slickcmd_common::consts::{WM_UIA_FOCUS_CHANGE, WM_WT_CONSOLE_ACTIVATE};
use slickcmd_common::{logd, win32};
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};
use std::{iter, mem, slice};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_core::implement;

#[derive(Clone)]
struct WtConsoleInfo {
    rt_id: String,
    hwnd: usize,
}

static CONSOLE_INFOS_MAP: LazyLock<Mutex<HashMap<usize, Vec<WtConsoleInfo>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new())); //keyed by wt hwnd

static CONSOLE_BOUNDS_MAP: LazyLock<Mutex<HashMap<usize, RECT>>> =
    LazyLock::new(|| Mutex::new(HashMap::new())); //keyed by console hwnd

static HWND_WT: AtomicUsize = AtomicUsize::new(0);
static HWND_CUR_CONSOLE: AtomicUsize = AtomicUsize::new(0);
static THREAD_ID: AtomicU32 = AtomicU32::new(0);

pub struct WtFocusMan<'a> {
    uia: &'a IUIAutomation,
    hwnd_wt: usize,
    focused_console_index: isize,
    console_infos: Vec<WtConsoleInfo>,
}

impl<'a> WtFocusMan<'a> {
    pub fn hwnd_wt() -> usize {
        HWND_WT.load(Ordering::Relaxed)
    }

    pub fn on_wt_activate(hwnd: usize) {
        HWND_WT.store(hwnd, Ordering::Relaxed);

        let (hthread, tid) = win32::create_thread(Some(wt_thread_proc), Some(hwnd as _));
        win32::close_handle(hthread);

        THREAD_ID.store(tid, Ordering::Relaxed);
    }

    pub fn on_wt_deactivate() {
        let tid_wt = THREAD_ID.load(Ordering::Relaxed);
        win32::post_thread_message(tid_wt, WM_QUIT, WPARAM(0), LPARAM(0));
        HWND_WT.store(0, Ordering::Relaxed);
    }

    pub fn get_console_bounds() -> RECT {
        let map = CONSOLE_BOUNDS_MAP.lock().unwrap();
        let hwnd_console = HWND_CUR_CONSOLE.load(Ordering::Relaxed);
        let bounds = map.get(&hwnd_console).copied();
        bounds.unwrap_or_default()
    }

    pub fn new(uia: &'a IUIAutomation, hwnd_wt: usize) -> WtFocusMan {
        let mut console_infos_map = CONSOLE_INFOS_MAP.lock().unwrap();
        let console_infos = console_infos_map.remove(&hwnd_wt).unwrap_or_default();
        WtFocusMan {
            uia,
            hwnd_wt,
            focused_console_index: -1,
            console_infos,
        }
    }

    pub fn init(&mut self) {
        self.check_focus();
    }

    fn check_focus(&mut self) {
        let focus_el = unsafe { self.uia.GetFocusedElement() };
        if focus_el.is_err() {
            self.focused_console_index = -1;
            self.notify_console_activate(0);
            return;
        }
        let focus_el = focus_el.unwrap();
        let bs_class_name = unsafe { focus_el.CurrentClassName() };
        let class_name = bs_class_name.as_ref().map(BSTR::to_string).unwrap_or_default();
        if class_name != "TermControl" {
            self.focused_console_index = -1;
            self.notify_console_activate(0);
        } else {
            let prev_console_index = self.focused_console_index;
            self.focused_console_index = self.set_focused_console(&focus_el);
            if prev_console_index != self.focused_console_index {
                if self.focused_console_index == -1 {
                    self.notify_console_activate(0);
                }
                else {
                    let info = &self.console_infos[self.focused_console_index as usize];
                    self.notify_console_activate(info.hwnd);
                }
            }
        }
    }

    fn notify_console_activate(&self, hwnd: usize) {
        let lparam = LPARAM(0);
        win32::send_message(
            GLOBAL.hwnd_msg(),
            WM_WT_CONSOLE_ACTIVATE,
            WPARAM(hwnd),
            lparam,
        );
    }

    pub fn get_rt_id(&self) -> String {
        if self.focused_console_index == -1 {
            return String::new();
        }
        let console = &self.console_infos[self.focused_console_index as usize];
        console.rt_id.clone()
    }

    fn save_console_bounds(&self, hwnd: usize, el: &IUIAutomationElement) {
        logd!("@ save console bounds");
        let bounds = unsafe { el.CurrentBoundingRectangle() }.unwrap_or_default();
        let mut map = CONSOLE_BOUNDS_MAP.lock().unwrap();

        let mut rc_wt = RECT::default();
        win32::get_window_rect(HWND(self.hwnd_wt as _), &mut rc_wt);

        let mut rect = RECT::default();
        rect.left = bounds.left - rc_wt.left;
        rect.top = bounds.top - rc_wt.top;
        rect.right = rect.left + bounds.right - bounds.left;
        rect.bottom = rect.top + bounds.bottom - bounds.top;
        map.insert(hwnd, rect);
    }

    fn set_focused_console(&mut self, el: &IUIAutomationElement) -> isize {
        let psa_rt_id = unsafe { el.GetRuntimeId() }.unwrap();
        let rt_id = rt_id_to_string(psa_rt_id);
        win32::safe_array_destroy(psa_rt_id);

        for (index, console) in self.console_infos.iter().enumerate() {
            if console.rt_id == rt_id {
                self.save_console_bounds(console.hwnd, el);
                HWND_CUR_CONSOLE.store(console.hwnd, Ordering::Relaxed);
                return index as _;
            }
        }
        let hwnd = self.resolve_console_hwnd(el);
        if hwnd == 0 {
            return -1;
        }

        self.save_console_bounds(hwnd, el);
        self.console_infos.push(WtConsoleInfo { rt_id, hwnd });
        HWND_CUR_CONSOLE.store(hwnd, Ordering::Relaxed);
        (self.console_infos.len() - 1) as _
    }

    fn resolve_console_hwnd(&self, el: &IUIAutomationElement) -> usize {
        let known_hwnds: HashSet<usize> = self.console_infos.iter().map(|x| x.hwnd).collect();

        let mut hwnds: Vec<usize> = Vec::new();
        let mut hwnd_after: HWND = HWND::default();
        let mut hwnd: HWND;
        loop {
            hwnd = win32::find_window_ex(
                HWND::default(),
                hwnd_after,
                Some("PseudoConsoleWindow"),
                None,
            );
            if hwnd.is_invalid() {
                break;
            }
            hwnd_after = hwnd;

            let hwnd_val = hwnd.0 as usize;
            if known_hwnds.contains(&hwnd_val) {
                continue;
            }
            if win32::get_parent(hwnd).0 as usize == self.hwnd_wt {
                hwnds.push(hwnd_val);
            }
        }
        if hwnds.is_empty() {
            logd!("something wrong..");
            return 0;
        }
        if hwnds.len() == 1 {
            return hwnds[0];
        }

        let mut pids: Vec<u32> = Vec::new();
        let mut ori_titles: Vec<String> = Vec::new();

        // Poor man's method for determining the focused console hwnd
        const CH_SPECIALS: [char; 3] = ['\u{00A0}', '\u{00AD}', '\u{200A}'];
        let mut ch_sp: char = '?';
        for ch_special in CH_SPECIALS {
            let mut ch_conflict = false;
            for (n, &hwnd) in hwnds.iter().enumerate() {
                let (pid, _) = win32::get_window_thread_process_id(HWND(hwnd as _));
                pids.push(pid);
                win32::attach_console(pid);
                let mut title = win32::get_console_title();
                if title.ends_with(ch_special) {
                    ch_conflict = true;
                    win32::free_console();
                    break;
                }
                ori_titles.push(title.clone());
                title.extend(iter::repeat(ch_special).take(n + 1));
                win32::set_console_title(&title);
                win32::free_console();
            }
            if !ch_conflict {
                ch_sp = ch_special;
                break;
            }
            for (n, ori_title) in ori_titles.iter().enumerate() {
                win32::attach_console(pids[n]);
                win32::set_console_title(&ori_title);
                win32::free_console();
            }
            ori_titles.clear();
        }
        if ch_sp == '?' {
            logd!("best tried, but failed to determine console hwnd");
            return 0;
        }
        let start_tick = win32::get_tick_count64();
        let focus_hwnd: usize = loop {
            let help_text = unsafe { el.GetCurrentPropertyValue(UIA_HelpTextPropertyId) };
            if help_text.is_err() {
                break 0;
            }
            let title = help_text.unwrap().to_string();
            let ori_title = title.trim_end_matches(ch_sp);
            if title != ori_title {
                let count = (title.len() - ori_title.len()) / ch_sp.to_string().len();
                break hwnds[count - 1];
            }
            win32::sleep(1);
            if win32::get_tick_count64() - start_tick > 1000 {
                break 0; //timed out
            }
        };
        for (n, &pid) in pids.iter().enumerate() {
            win32::attach_console(pid);
            win32::set_console_title(&ori_titles[n]);
            win32::free_console();
        }
        focus_hwnd
    }

    fn dispose(&mut self) {
        let mut console_infos = mem::replace(&mut self.console_infos, Vec::new());
        let mut invalid_console_indexes: Vec<usize> = Vec::new();
        for n in (0..console_infos.len()).rev() {
            let hwnd = console_infos[n].hwnd;
            let hwnd = HWND(hwnd as _);
            if !win32::is_window(hwnd) || win32::get_parent(hwnd).0 as usize != self.hwnd_wt {
                invalid_console_indexes.push(n);
            }
        }
        for n in invalid_console_indexes {
            console_infos.remove(n);
        }
        if !console_infos.is_empty() {
            let mut console_infos_map = CONSOLE_INFOS_MAP.lock().unwrap();
            console_infos_map.insert(self.hwnd_wt, console_infos);
        }
    }
}

fn rt_id_to_string(psa_rt_id: *const SAFEARRAY) -> String {
    let sa_rt_id: &SAFEARRAY = &unsafe { *psa_rt_id };
    let count = sa_rt_id.rgsabound[0].cElements as usize;
    let data = unsafe { slice::from_raw_parts(sa_rt_id.pvData as *const u8, count * 4) };
    let rt_id = faster_hex::hex_string(data);
    rt_id
}

#[implement(IUIAutomationFocusChangedEventHandler)]
#[derive(Default)]
struct UIAutomationFocusChangedEventHandler {
    tid_notify: u32,
    last_rt_id: Mutex<String>,
}

impl UIAutomationFocusChangedEventHandler {
    pub fn new(tid_notify: u32, focused_rt_id: String) -> UIAutomationFocusChangedEventHandler {
        UIAutomationFocusChangedEventHandler {
            tid_notify,
            last_rt_id: Mutex::new(focused_rt_id),
        }
    }
}

impl IUIAutomationFocusChangedEventHandler_Impl for UIAutomationFocusChangedEventHandler_Impl {
    fn HandleFocusChangedEvent(
        &self,
        sender: Option<&IUIAutomationElement>,
    ) -> windows_core::Result<()> {
        let mut rt_id = String::new();
        if let Some(el) = sender {
            let class_name = unsafe { el.CurrentClassName() };
            if class_name.is_ok() {
                let class_name = class_name.unwrap().to_string();
                if class_name == "TermControl" {
                    let psa_rt_id = unsafe { el.GetRuntimeId() }.unwrap();
                    rt_id = rt_id_to_string(psa_rt_id);
                    win32::safe_array_destroy(psa_rt_id);
                }
            }
        }

        let mut last_rt_id = self.last_rt_id.lock().unwrap();
        if !last_rt_id.eq(&rt_id) {
            *last_rt_id = rt_id;
            win32::post_thread_message(self.tid_notify, WM_UIA_FOCUS_CHANGE, WPARAM(0), LPARAM(0));
        }
        Ok(())
    }
}

extern "system" fn wt_thread_proc(lp_param: *mut c_void) -> u32 {
    let hwnd_wt = lp_param as usize;
    win32::co_initialize_ex(COINIT_MULTITHREADED);
    let uia: IUIAutomation = win32::co_create_instance(&CUIAutomation).unwrap();
    let mut wt_focus_man = WtFocusMan::new(&uia, hwnd_wt);

    wt_focus_man.init();

    let tid = win32::get_current_thread_id();
    let handler = UIAutomationFocusChangedEventHandler::new(tid, wt_focus_man.get_rt_id());
    let handler: IUIAutomationFocusChangedEventHandler = handler.into();
    _ = unsafe { uia.AddFocusChangedEventHandler(None, &handler) };

    let mut msg: MSG = MSG::default();
    let hwnd_none = HWND::default();
    win32::set_timer(HWND::default(), 1, 2000, None);
    let mut prev_focus_el = unsafe { uia.GetFocusedElement() }.ok();
    loop {
        if !win32::get_message(&mut msg, hwnd_none) {
            break;
        }
        if msg.message == WM_TIMER {
            if WtFocusMan::hwnd_wt() != hwnd_wt {
                break;
            }
            let focus_el = unsafe { uia.GetFocusedElement() }.ok();
            let focus_changed = if prev_focus_el.is_none() || focus_el.is_none() {
                prev_focus_el.is_some() || focus_el.is_some()
            } else {
                let a = prev_focus_el.as_ref().unwrap();
                let b = focus_el.as_ref().unwrap();
                unsafe { uia.CompareElements(a, b) }.unwrap_or_default().as_bool() == false
            };
            if focus_changed {
                prev_focus_el = focus_el;
                wt_focus_man.check_focus();
            }
            wt_focus_man.check_focus();
        } else if msg.message == WM_UIA_FOCUS_CHANGE {
            wt_focus_man.check_focus();
        }
    }
    wt_focus_man.dispose();
    _ = unsafe { uia.RemoveFocusChangedEventHandler(&handler) };
    0u32
}