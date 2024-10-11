use crate::ac_list::AC_LIST;
use crate::app_comm::APP_COMM;
use crate::msg_win::MsgWin;
use crate::GLOBAL;
use slickcmd_common::{logd, win32};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Core {
    msg_win: MsgWin,
    hhook: HHOOK,
}

unsafe impl Send for Core {}
unsafe impl Sync for Core {}

impl Core {
    pub const fn new() -> Core {
        unsafe { core::mem::zeroed() }
    }

    pub fn attach(&mut self, hwnd_target: HWND, kbdproc: HOOKPROC) -> LRESULT {
        GLOBAL.set_hwnd_target(hwnd_target);
        GLOBAL.set_suppress_hook(false);

        APP_COMM.init();

        if win32::get_prop(hwnd_target, "SLCK_CMD_ATTACHED").0 as usize != 0 {
            logd!("(already attached)");
            return LRESULT(0);
        }

        self.msg_win.create();

        let hhook = win32::set_windows_hook_ex(WH_KEYBOARD, kbdproc, GLOBAL.hinstance(), 0);
        if hhook.is_invalid() {
            logd!("keyboard hook install failed.");
            return LRESULT(1);
        }

        self.hhook = hhook;
        GLOBAL.set_hhook(hhook);

        win32::set_prop(hwnd_target, "SLCK_CMD_ATTACHED", HANDLE(1 as _));
        logd!("@ core attached.");

        LRESULT(0)
    }

    pub fn detach(&mut self) {
        win32::unhook_widows_hook_ex(GLOBAL.hhook());
        win32::remove_prop(GLOBAL.hwnd_target(), "SLCK_CMD_ATTACHED");
        logd!("@ core detached.");
    }

    pub fn kbd_proc(&mut self, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let vk = wparam.0 as u16;
        let dw_lparam = lparam.0 as u32;
        let alt_down = (dw_lparam & 0x20000000) != 0;
        let key_up = (dw_lparam & 0x80000000) != 0;

        if GLOBAL.suppress_hook() {
            if vk == VK_F12.0 {
                if key_up {
                    GLOBAL.set_suppress_hook(false);
                }
                return LRESULT(1);
            }
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }

        if GLOBAL.showing_menu() {
            return unsafe { CallNextHookEx(None, code, wparam, lparam) };
        }

        if GLOBAL.showing_acl() {
            let acl = AC_LIST.lock().unwrap();
            if vk == VK_UP.0
                || vk == VK_DOWN.0
                || vk == VK_PRIOR.0
                || vk == VK_NEXT.0
                || vk == VK_ESCAPE.0
                || vk == VK_TAB.0
            {
                let msg = if key_up { WM_KEYUP } else { WM_KEYDOWN };
                let ret =win32::send_message(acl.hwnd, msg, WPARAM(vk as _), LPARAM(0));
                if ret.0 == 0 {
                    return LRESULT(1);
                }
            }
            if vk == VK_RETURN.0 {
                acl.close();
            }
        }

        if key_up {
            if APP_COMM.process_key_up(vk, alt_down) {
                return LRESULT(1);
            }
        } else {
            if alt_down && GLOBAL.showing_acl() {
                AC_LIST.lock().unwrap().close();
            }
            if vk == VK_RETURN.0 {
                if APP_COMM.process_key_down(vk, alt_down) {
                    return LRESULT(1);
                }
            }
        }

        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }
}
