use slickcmd_common::{logd, win32};
use slickcmd_common::win32::get_last_error;
use std::mem::size_of;
use windows::{
    core::GUID,
    Win32::{Foundation::*, UI::Shell::*, UI::WindowsAndMessaging::*},
};

#[derive(Default)]
pub struct TrayIcon {
    hwnd_callback: HWND,

    pub guid: GUID,
}

impl TrayIcon {
    pub fn create(
        &mut self,
        hicon: HICON,
        tooltip: &str,
        hwnd_callback: HWND,
        callback_msg: u32,
        guid: &GUID,
        id: u32,
    ) -> WIN32_ERROR {
        self.guid = *guid;
        self.hwnd_callback = hwnd_callback;

        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            uFlags: NIF_ICON
                | NIF_TIP
                | NIF_MESSAGE
                | NIF_STATE
                | NIF_GUID
                | NIF_SHOWTIP
                | NIF_INFO,
            dwInfoFlags: NIIF_NOSOUND,
            hIcon: hicon,
            guidItem: self.guid,
            uID: id,
            hWnd: self.hwnd_callback,
            uCallbackMessage: callback_msg,
            ..Default::default()
        };
        let wsz_tooltip = win32::wsz_from_str(tooltip);
        nid.szTip[..wsz_tooltip.len()].copy_from_slice(wsz_tooltip.as_slice());

        let wsz_info = win32::wsz_from_str("Slick Cmd Started");
        nid.szInfo[..wsz_info.len()].copy_from_slice(wsz_info.as_slice());

        if !win32::shell_notify_icon(NIM_ADD, &nid) {
            return get_last_error();
        }
        nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        if !win32::shell_notify_icon(NIM_SETVERSION, &nid) {
            return get_last_error();
        }
        NO_ERROR
    }

    pub fn destroy(&mut self) {
        let nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            uFlags: NIF_GUID,
            guidItem: self.guid,
            ..Default::default()
        };
        if !win32::shell_notify_icon(NIM_DELETE, &nid) {
            logd!("delete notify icon failed.");
        }
    }
}
