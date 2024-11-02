use slickcmd_common::{consts, win32};
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{HWND_MESSAGE, WM_SETTEXT};

#[derive(Default)]
pub struct AppComm {
    hwnd_app_msg: AtomicUsize,
}

pub static APP_COMM: AppComm = AppComm::new();

impl AppComm {
    const fn new() -> AppComm {
        AppComm {
            hwnd_app_msg: AtomicUsize::new(0),
        }
    }

    fn hwnd(&self) -> HWND {
        HWND(self.hwnd_app_msg.load(Ordering::Relaxed) as _)
    }

    fn set_hwnd(&self, hwnd: HWND) {
        self.hwnd_app_msg.store(hwnd.0 as usize, Ordering::Relaxed);
    }

    pub fn init(&self) {
        let hwnd = win32::find_window_ex(HWND_MESSAGE, HWND::default(), Some("slck_cmd_msg"), None);
        self.set_hwnd(hwnd);
    }

    pub fn process_key_down(&self, vk: u16, alt_down: bool) -> bool {
        let wparam = WPARAM(vk as usize);
        let lparam = if alt_down { LPARAM(1) } else { LPARAM(0) };
        win32::send_message(self.hwnd(), consts::WM_CORE_KEYDOWN, wparam, lparam).0 != 0
    }

    pub fn process_key_up(&self, vk: u16, alt_down: bool) -> bool {
        let wparam = WPARAM(vk as usize);
        let lparam = if alt_down { LPARAM(1) } else { LPARAM(0) };
        win32::send_message(self.hwnd(), consts::WM_CORE_KEYUP, wparam, lparam).0 != 0
    }

    #[allow(dead_code)]
    pub fn notify_ac_list_closed(&self) {
        win32::send_message(
            self.hwnd(),
            consts::WM_NOTIFY_AC_LIST_CLOSED,
            WPARAM(0),
            LPARAM(0),
        );
    }

    #[allow(dead_code)]
    pub fn update_input(&self, input: &str) {
        let wsz = win32::wsz_from_str(input);
        win32::send_message(
            self.hwnd(),
            WM_SETTEXT,
            WPARAM(1),
            LPARAM(wsz.as_ptr() as isize),
        );
    }
}
