use std::sync::Mutex;
use SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::Foundation::*;
use windows::Win32::System::SystemServices;
use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, HC_ACTION};

use crate::core::Core;
use crate::global::Global;

mod core;
mod app_comm;
mod msg_win;
mod global;
mod ac_list;

static CORE: Mutex<Core> = Mutex::new(Core::new());
static GLOBAL: Global = Global::new();

#[no_mangle]
#[allow(non_snake_case)]
fn DllMain(hinstance: HINSTANCE, dw_reason: u32, _: usize) -> BOOL {
    match dw_reason {
        DLL_PROCESS_ATTACH => {
            if !GLOBAL.init(hinstance) {
                return FALSE;
            }
        },
        _ => (),
    }
    TRUE
}

#[allow(non_snake_case)]
extern "system" fn KbdProc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code != HC_ACTION as i32 {
        return unsafe{CallNextHookEx(None, code, wparam, lparam)};
    }
    CORE.lock().unwrap().kbd_proc(code, wparam, lparam)
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn Attach(hwnd_target: HWND) -> LRESULT {
    CORE.lock().unwrap().attach(hwnd_target, Some(KbdProc))
}

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn Detach() {
    CORE.lock().unwrap().detach();
}
