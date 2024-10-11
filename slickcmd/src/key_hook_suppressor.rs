use slickcmd_common::{consts::WM_SUPPRESS_CORE_KEY, logd, win32};
use windows::Win32::{
    Foundation::*,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

pub struct KeyHookSuppressor {
    hwnd_target: HWND,
}

impl KeyHookSuppressor {
    pub fn new(hwnd_target: HWND) -> KeyHookSuppressor {
        logd!("@new keyhook suppressor");
        let hwnd = win32::find_window_ex(HWND_MESSAGE, None, Some("slck_cmd_core_msg"), None);
        win32::send_message(hwnd, WM_SUPPRESS_CORE_KEY, WPARAM(0), LPARAM(0));
        KeyHookSuppressor { hwnd_target }
    }
}

impl Drop for KeyHookSuppressor {
    fn drop(&mut self) {
        if win32::get_foreground_window() != self.hwnd_target {
            return;
        }
        let mut inputs = [INPUT::default(); 2];
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki.wVk = VK_F12;

        inputs[1] = inputs[0];
        inputs[1].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        win32::send_input(&inputs);
    }
}
