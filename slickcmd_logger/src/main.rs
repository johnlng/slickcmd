use std::mem::size_of;
use std::process::Command;

use slickcmd_common::win32;
use widestring::U16CStr;
use windows::core::imp::PCWSTR;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, DispatchMessageW, GetMessageW, HMENU, HWND_MESSAGE, MSG, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_SETTEXT, WNDCLASSEXW};

fn main() {
    let _hmutex = win32::create_mutex(false, "slck_cmd_logger");
    if win32::get_last_error() == ERROR_ALREADY_EXISTS {
        return;
    }

    // println!("Hello logger..");

    //
    let window_class = "slck_cmd_log";
    let wsz_class = win32::wsz_from_str(window_class);

    let hinstance: HINSTANCE = win32::get_module_handle().into();

    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        lpszClassName: win32::pwsz(&wsz_class),
        hInstance: hinstance,
        lpfnWndProc: Some(s_wndproc),
        ..Default::default()
    };

    let atom = win32::register_class_ex(&wc);
    debug_assert!(atom != 0);

    let hwnd = win32::create_window_ex(
        WINDOW_EX_STYLE::default(),
        window_class,
        "",
        WINDOW_STYLE::default(),
        0, 0, 0, 0,
        HWND_MESSAGE,
        HMENU::default(),
        hinstance,
        None
    );
    assert!(!hwnd.is_invalid());

    _= Command::new("cmd")
        .args(["/C", "title", "slickcmd", "Logger"]).spawn();

    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).into() {
            _= TranslateMessage(&msg);
            _= DispatchMessageW(&msg);
        }
    }
}

extern "system" fn s_wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {

    if message == WM_SETTEXT && lparam.0 != 0 {
        let pwsz_msg = PCWSTR::from(lparam.0 as *const u16);
        let wsz_msg = unsafe { U16CStr::from_ptr_str(pwsz_msg) };
        let msg = wsz_msg.to_string_lossy();
        if msg == "::CLS" {
            _= Command::new("cmd").args(["/c", "cls"]).status();
        }
        else {
            println!("{}", msg);
        }
        return LRESULT(0);
    }

    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}