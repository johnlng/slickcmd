mod global;

use crate::global::Global;
use slickcmd_common::{logd, win32};
use std::ffi::c_void;
use std::path::Path;
use windows::Win32::{Foundation::*, System::SystemServices::*, UI::WindowsAndMessaging::*};

static GLOBAL: Global = Global::new();

#[no_mangle]
#[allow(non_snake_case)]
fn DllMain(hinstance: HINSTANCE, dw_reason: u32, _: usize) -> BOOL {
    match dw_reason {
        DLL_PROCESS_ATTACH => {
            if !GLOBAL.init(hinstance) {
                return FALSE;
            }
        }
        _ => (),
    }
    TRUE
}

#[no_mangle]
extern "system" fn ShlProc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    if code != HSHELL_WINDOWACTIVATED as i32 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    if cfg!(debug_assertions) {
        let exe_path = win32::get_module_file_name(HMODULE::default());
        let exe_name = Path::new(&exe_path)
            .file_name()
            .unwrap_or_default()
            .to_ascii_lowercase();

        // logd!("exe: {}", exe_name.to_string_lossy().to_string());

        if win32::is_debugger_present()
            || exe_name == "rustrover64.exe"
            || exe_name == "code.exe"
            || exe_name == "slickcmd_logger.exe"
        {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }
    }

    logd!("@ ShlProc: {:x} activated.", wparam.0);

    let hwnd = HWND(wparam.0 as *mut c_void);

    let mut hwnd_msg = GLOBAL.hwnd_msg();
    let shl_msg = GLOBAL.shl_msg();

    if hwnd_msg.is_invalid() {
        hwnd_msg = win32::find_window_ex(HWND_MESSAGE, HWND::default(), Some("slck_cmd_msg"), None);
        GLOBAL.set_hwnd_msg(hwnd_msg);
    }
    if !hwnd_msg.is_invalid() {
        logd!("app msg found.");

        let dw_hwnd = wparam.0 as u32;
        let mut b_attach_core = false;

        let ret = win32::send_message(hwnd_msg, shl_msg, wparam, LPARAM::default());
        if (ret.0 >> 32) as u32 == dw_hwnd {
            b_attach_core = (ret.0 as u32) == 1
        } else {
            // using obsolete hwnd_msg?
            hwnd_msg = win32::find_window_ex(HWND_MESSAGE, HWND::default(), Some("slck_cmd_msg"), None);
            if !hwnd_msg.is_invalid() {
                let ret = win32::send_message(hwnd_msg, shl_msg, wparam, LPARAM::default());
                if (ret.0 >> 32) as u32 != dw_hwnd {
                    // something wrong?
                    hwnd_msg = HWND::default();
                } else {
                    b_attach_core = (ret.0 as u32) == 1
                }
            }
            GLOBAL.set_hwnd_msg(hwnd_msg);
        }

        if b_attach_core {
            logd!("attach core.");
            attach_core(hwnd);
        }
    } else {
        logd!("app msg not found?");
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

type FnAttach = extern "system" fn(hwnd: HWND) -> LRESULT;

fn attach_core(hwnd: HWND) {
    let dll_path = win32::get_module_file_name(GLOBAL.hinstance().into());
    let dll_path = Path::new(&dll_path);
    let core_dll_path = dll_path.parent().unwrap().join("slickcmd_core.dll");
    let hmod_core = win32::load_library(&core_dll_path.to_string_lossy());

    if let Some(hmod_core) = hmod_core {
        let p = win32::get_proc_address(hmod_core, "Attach");
        let fn_attach: FnAttach = unsafe { std::mem::transmute(p) };
        fn_attach(hwnd);
    }
}
