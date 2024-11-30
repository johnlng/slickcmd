use crate::global::GLOBAL;
use crate::tray_icon::TrayIcon;
use slickcmd_common::consts::{WM_TRAY_CALLBACK};
use slickcmd_common::{utils, win32};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::LazyLock;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{
    GCLP_HICONSM, HICON, ICON_SMALL, IDI_APPLICATION, SW_HIDE, SW_SHOWNORMAL, WM_GETICON,
};
use windows_core::GUID;

#[derive(Clone)]
struct TrayWin {
    hwnd: HWND,

    guid: GUID,
    tray_icon: TrayIcon,
}

#[derive(Default)]
pub struct TrayWins(RefCell<Vec<Option<Rc<TrayWin>>>>);

unsafe impl Sync for TrayWins {}
unsafe impl Send for TrayWins {}

impl TrayWins {
    pub fn add(hwnd: HWND) {
        let mut v = TRAY_WINS.0.borrow_mut();
        let mut guids: HashSet<GUID> = HashSet::new();
        let mut index = v.len();
        for (n, item) in v.iter().enumerate() {
            if item.is_none() {
                index = n;
            }
            else {
                guids.insert(item.as_ref().unwrap().guid);
            }
        }
        let win = Some(TrayWin::new(hwnd, index, guids));
        if index == v.len() {
            v.push(win);
        } else {
            v[index] = win;
        }
    }

    pub fn restore(index: usize) {
        let mut v = TRAY_WINS.0.borrow_mut();
        v[index].as_ref().unwrap().restore();
        v[index] = None;
    }

    pub fn restore_all() {
        let mut v = TRAY_WINS.0.borrow_mut();
        for item in v.iter() {
            if let Some(item) = item {
                item.restore();
            }
        }
        v.clear();
    }
}

static TRAY_WINS: LazyLock<TrayWins> = LazyLock::new(|| TrayWins::default());

impl TrayWin {
    fn new(hwnd: HWND, index: usize, existing_guids: HashSet<GUID>) -> Rc<TrayWin> {
        win32::show_window(hwnd, SW_HIDE);
        let ret = win32::send_message(hwnd, WM_GETICON, WPARAM(ICON_SMALL as _), LPARAM(0));
        let mut h_icon = HICON(ret.0 as _);
        if h_icon.is_invalid() {
            let ret = win32::get_class_long_ptr(hwnd, GCLP_HICONSM);
            h_icon = HICON(ret as _);
        }
        if h_icon.is_invalid() {
            h_icon = win32::load_icon(HINSTANCE::default(), IDI_APPLICATION.0 as _);
        }

        let title = win32::get_window_text(hwnd);
        let (mut pid, _) = win32::get_window_thread_process_id(hwnd);
        let new_attach = win32::attach_console(pid);
        let mut pids = [0u32; 4];
        let count = win32::get_console_process_list(&mut pids) as usize;
        let cur_pid = win32::get_current_process_id();
        for n in 0..count {
            if pids[n] != pid && pids[n] != cur_pid {
                pid = pids[n];
                break;
            }
        }
        if new_attach {
            win32::free_console();
        }
        let h_proc = win32::open_process(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);
        let exe_path = win32::get_module_file_name_ex(h_proc).to_string_lossy().into_owned();
        win32::close_handle(h_proc);

        let mut guid = GUID::default();
        for n in 0..1024 {
            let content = format!("{}|{}|{}", exe_path, title, n);
            let md5 = md5::compute(content.as_bytes());
            guid = utils::u8s_as_guid(&md5.0);
            if !existing_guids.contains(&guid) {
                break;
            }
        }

        let mut tray_icon = TrayIcon::default();
        let _ = tray_icon.create(
            h_icon,
            &title,
            "",
            GLOBAL.hwnd_msg(),
            WM_TRAY_CALLBACK,
            &guid,
            index as _,
        );

        let win = Rc::new(TrayWin {
            hwnd,
            guid,
            tray_icon,
        });

        win
    }

    fn restore(&self) {
        self.tray_icon.destroy();
        win32::show_window(self.hwnd, SW_SHOWNORMAL);
        win32::set_foreground_window(self.hwnd);
    }
}
