use crate::log;
use std::ffi::{c_void, OsString};
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use widestring::{U16CStr, U16CString};
use windows::core::*;
use windows::Wdk::System::Threading::*;
use windows::Win32::Foundation::*;
use windows::Win32::Globalization::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Environment::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Ole::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::SystemInformation::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::WindowsProgramming::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub fn create_mutex(initial_owner: bool, name: &str) -> HANDLE {
    let wsz_name = wsz_from_str(name);
    unsafe { CreateMutexW(None, initial_owner, pwsz(&wsz_name)).unwrap() }
}

pub fn get_last_error() -> WIN32_ERROR {
    unsafe { GetLastError() }
}

pub fn get_module_handle() -> HMODULE {
    unsafe { GetModuleHandleW(None).unwrap() }
}

pub fn wsz_from_str(str: &str) -> U16CString {
    unsafe { U16CString::from_str_unchecked(str) }
}

pub fn pwsz(wsz: &U16CString) -> PCWSTR {
    PCWSTR::from_raw(wsz.as_ptr())
}

pub fn lparam_as_ref<T>(lparam: &LPARAM) -> &T {
    unsafe { &*(lparam.0 as *const T) }
}

pub fn set_window_long_ptr<T>(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, p: *const T) {
    unsafe { SetWindowLongPtrW(hwnd, index, p as isize) };
}

pub fn get_window_long_ptr<T>(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> *const T {
    unsafe { GetWindowLongPtrW(hwnd, index) as *const T }
}

pub fn get_window_long_ptr_mut<T>(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> *mut T {
    unsafe { GetWindowLongPtrW(hwnd, index) as *mut T }
}

pub fn dialog_box(hinstance: HINSTANCE, res_id: u16, hwnd_parent: HWND, dlgproc: DLGPROC) -> isize {
    unsafe {
        DialogBoxParamW(
            hinstance,
            PCWSTR(res_id as *const u16),
            hwnd_parent,
            dlgproc,
            LPARAM(0),
        )
    }
}

pub fn dialog_box_param(
    hinstance: HINSTANCE,
    res_id: u16,
    hwnd_parent: HWND,
    dlgproc: DLGPROC,
    initparam: LPARAM,
) -> isize {
    unsafe {
        DialogBoxParamW(
            hinstance,
            PCWSTR(res_id as *const u16),
            hwnd_parent,
            dlgproc,
            initparam,
        )
    }
}

pub fn destroy_window(hwnd: HWND) {
    unsafe {
        _ = DestroyWindow(hwnd);
    }
}

pub fn post_quit_message(exit_code: i32) {
    unsafe { PostQuitMessage(exit_code) }
}

pub fn end_dialog(hwnd: HWND, result: MESSAGEBOX_RESULT) {
    unsafe {
        _ = EndDialog(hwnd, result.0 as _);
    }
}

pub fn load_icon(hinstance: HINSTANCE, res_id: u16) -> HICON {
    unsafe { LoadIconW(hinstance, PCWSTR(res_id as *const u16)).unwrap() }
}

pub fn load_cursor(res_id: PCWSTR) -> HCURSOR {
    unsafe { LoadCursorW(None, res_id).unwrap() }
}

pub fn register_class_ex(wc: &WNDCLASSEXW) -> u16 {
    unsafe { RegisterClassExW(wc) }
}

pub fn system_parameters_info<T>(
    uiaction: SYSTEM_PARAMETERS_INFO_ACTION,
    uiparam: u32,
    pvparam: *mut T,
) {
    unsafe {
        let _ = SystemParametersInfoW(
            uiaction,
            uiparam,
            Some(pvparam as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        );
    }
}

pub fn adjust_window_rect(rc_win: &mut RECT, style: WINDOW_STYLE, bmenu: bool) {
    unsafe {
        _ = AdjustWindowRect(rc_win, style, bmenu);
    }
}

pub fn create_window_ex(
    ex_style: WINDOW_EX_STYLE,
    class_name: &str,
    window_name: &str,
    style: WINDOW_STYLE,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    hwnd_parent: HWND,
    hmenu: HMENU,
    hinstance: HINSTANCE,
    lp_param: Option<*const c_void>,
) -> HWND {
    let wsz_class_name = wsz_from_str(class_name);
    let wsz_window_name = wsz_from_str(window_name);

    unsafe {
        CreateWindowExW(
            ex_style,
            pwsz(&wsz_class_name),
            pwsz(&wsz_window_name),
            style,
            x,
            y,
            w,
            h,
            hwnd_parent,
            hmenu,
            hinstance,
            lp_param,
        )
        .unwrap()
    }
}

pub fn show_window(hwnd: HWND, show_cmd: SHOW_WINDOW_CMD) {
    unsafe {
        _ = ShowWindow(hwnd, show_cmd);
    }
}

pub fn shell_notify_icon(message: NOTIFY_ICON_MESSAGE, data: &NOTIFYICONDATAW) -> bool {
    unsafe { Shell_NotifyIconW(message, data as *const NOTIFYICONDATAW).into() }
}

pub fn close_handle(handle: HANDLE) {
    unsafe {
        _ = CloseHandle(handle);
    }
}

pub fn load_accelerators(hinstance: HINSTANCE, res_id: u16) -> HACCEL {
    unsafe { LoadAcceleratorsW(hinstance, PCWSTR(res_id as *const u16)).unwrap() }
}

//
pub fn register_window_message(name: &str) -> u32 {
    let mut wsz_name: Vec<u16> = name.encode_utf16().collect();
    wsz_name.push(0);
    unsafe { RegisterWindowMessageW(PCWSTR::from_raw(wsz_name.as_ptr())) }
}

pub fn get_module_file_name_ex(hprocess: HANDLE) -> OsString {
    const BUF_SIZE: usize = MAX_PATH as _;
    let mut buf: [u16; BUF_SIZE] = [0; BUF_SIZE];
    let cch = unsafe { GetModuleFileNameExW(hprocess, HMODULE::default(), &mut buf) };
    OsString::from_wide(&buf[..cch as usize])
}

pub fn get_module_file_name(hmodule: HMODULE) -> OsString {
    const BUF_SIZE: usize = MAX_PATH as _;
    let mut buf: [u16; BUF_SIZE] = [0; BUF_SIZE];
    let cch = unsafe { GetModuleFileNameW(hmodule, &mut buf) };
    OsString::from_wide(&buf[..cch as usize])
}

pub fn find_window_ex(
    hwnd_parent: HWND,
    hwnd_after: Option<HWND>,
    class_name: Option<&str>,
    window_name: Option<&str>,
) -> HWND {
    let pwsz_class_name: PCWSTR;
    let mut wsz_class_name: Vec<u16>;
    if let Some(class_name) = class_name {
        wsz_class_name = class_name.encode_utf16().collect();
        wsz_class_name.push(0);
        pwsz_class_name = PCWSTR::from_raw(wsz_class_name.as_ptr());
    } else {
        pwsz_class_name = PCWSTR::null();
    }

    let pwsz_window_name: PCWSTR;
    let mut wsz_window_name: Vec<u16>;
    if let Some(window_name) = window_name {
        wsz_window_name = window_name.encode_utf16().collect();
        wsz_window_name.push(0);
        pwsz_window_name = PCWSTR::from_raw(wsz_window_name.as_ptr());
    } else {
        pwsz_window_name = PCWSTR::null();
    }

    unsafe {
        FindWindowExW(
            hwnd_parent,
            hwnd_after.as_ref(),
            pwsz_class_name,
            pwsz_window_name,
        )
        .unwrap_or_default()
    }
}

pub fn send_message(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { SendMessageW(hwnd, msg, wparam, lparam) }
}

pub fn post_message(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) {
    unsafe {
        _ = PostMessageW(hwnd, msg, wparam, lparam);
    }
}

pub fn load_library(path: &str) -> Option<HMODULE> {
    let mut wsz_path: Vec<u16> = path.encode_utf16().collect();
    wsz_path.push(0);

    unsafe { LoadLibraryW(PCWSTR::from_raw(wsz_path.as_ptr())).ok() }
}

pub fn get_proc_address(hmodule: HMODULE, name: &str) -> *const c_void {
    unsafe {
        let mut s_name = String::from(name);
        if s_name.chars().last().unwrap() != '\0' {
            s_name.push('\0');
        }
        let fp = GetProcAddress(hmodule, PCSTR::from_raw(s_name.as_ptr())).unwrap();
        std::mem::transmute(fp)
    }
}

pub fn get_prop(hwnd: HWND, name: &str) -> HANDLE {
    let wsz_name = wsz_from_str(name);
    unsafe { GetPropW(hwnd, PCWSTR::from_raw(wsz_name.as_ptr())) }
}

pub fn set_prop(hwnd: HWND, name: &str, data: HANDLE) {
    let wsz_name = wsz_from_str(name);
    unsafe {
        _ = SetPropW(hwnd, pwsz(&wsz_name), data);
    }
}

pub fn get_window_thread_process_id(hwnd: HWND) -> (u32, u32) {
    let mut pid: u32 = 0;
    let tid = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32)) };
    (pid, tid)
}

pub fn set_windows_hook_ex(
    hook_id: WINDOWS_HOOK_ID,
    lpfn: HOOKPROC,
    hmod: HINSTANCE,
    thread_id: u32,
) -> HHOOK {
    unsafe { SetWindowsHookExW(hook_id, lpfn, hmod, thread_id).unwrap() }
}

pub fn is_window(hwnd: HWND) -> bool {
    unsafe { IsWindow(hwnd).into() }
}

pub fn is_iconic(hwnd: HWND) -> bool {
    unsafe { IsIconic(hwnd).into() }
}

pub fn get_async_key_state(vk: VIRTUAL_KEY) -> i16 {
    unsafe { GetAsyncKeyState(vk.0 as i32) }
}

pub fn get_key_state(vk: VIRTUAL_KEY) -> i16 {
    unsafe { GetKeyState(vk.0 as i32) }
}

pub fn get_class_name(hwnd: HWND) -> String {
    const BUF_SIZE: usize = MAX_PATH as _;
    let mut buf = [0u16; BUF_SIZE];
    let cch = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..cch as usize])
}

pub fn create_popup_menu() -> HMENU {
    unsafe { CreatePopupMenu().unwrap() }
}

pub fn append_menu(hmenu: HMENU, flags: MENU_ITEM_FLAGS, id: u16, text: Option<&str>) -> bool {
    let mut pwsz_text: PCWSTR = PCWSTR::null();
    let wsz_text: U16CString;
    if let Some(text) = text {
        wsz_text = wsz_from_str(text);
        pwsz_text = pwsz(&wsz_text);
    }
    unsafe { AppendMenuW(hmenu, flags, id as usize, pwsz_text).is_ok() }
}

pub fn append_sub_menu(hmenu: HMENU, hmenu_popup: HMENU, text: &str) -> bool {
    let wsz_text = wsz_from_str(text);
    let pwsz_text = pwsz(&wsz_text);
    unsafe { AppendMenuW(hmenu, MF_STRING | MF_POPUP, hmenu_popup.0 as _, pwsz_text).is_ok() }
}

pub fn set_foreground_window(hwnd: HWND) {
    unsafe {
        _ = SetForegroundWindow(hwnd);
    }
}

pub fn set_focus(hwnd: HWND) {
    unsafe {
        _ = SetFocus(hwnd);
    }
}

pub fn track_popup_menu(
    hmenu: HMENU,
    flags: TRACK_POPUP_MENU_FLAGS,
    x: i32,
    y: i32,
    hwnd: HWND,
) -> i32 {
    unsafe { TrackPopupMenu(hmenu, flags, x, y, 0, hwnd, None).0 }
}

pub fn destroy_menu(hmenu: HMENU) {
    unsafe {
        _ = DestroyMenu(hmenu);
    }
}

pub fn unhook_widows_hook_ex(hhook: HHOOK) {
    unsafe {
        _ = UnhookWindowsHookEx(hhook);
    }
}

pub fn get_class_long_ptr(hwnd: HWND, index: GET_CLASS_LONG_INDEX) -> usize {
    unsafe { GetClassLongPtrW(hwnd, index) }
}

pub fn get_window_text(hwnd: HWND) -> String {
    unsafe {
        let cch = GetWindowTextLengthW(hwnd);
        let mut buf = vec![0u16; (cch + 1) as usize];
        let cch = GetWindowTextW(hwnd, buf.as_mut_slice());
        String::from_utf16_lossy(&buf[..cch as usize])
    }
}

pub fn open_process(
    desired_access: PROCESS_ACCESS_RIGHTS,
    inherit_handle: bool,
    pid: u32,
) -> HANDLE {
    unsafe { OpenProcess(desired_access, inherit_handle, pid).unwrap_or_default() }
}

pub fn register_hotkey(hwnd: HWND, id: i32, modifiers: HOT_KEY_MODIFIERS, vk: u32) -> bool {
    unsafe { RegisterHotKey(hwnd, id, modifiers, vk).is_ok() }
}

pub fn unregister_hotkey(hwnd: HWND, id: i32) {
    unsafe {
        _ = UnregisterHotKey(hwnd, id);
    }
}

pub fn nt_query_information_process(
    hprocess: HANDLE,
    infoclass: PROCESSINFOCLASS,
    pinfo: *mut c_void,
    info_len: u32,
) -> NTSTATUS {
    let mut len_return: u32 = 0;
    unsafe { NtQueryInformationProcess(hprocess, infoclass, pinfo, info_len, &mut len_return) }
}

pub fn read_process_memory(
    hproc: HANDLE,
    base_addr: *const c_void,
    lpbuf: *mut c_void,
    size: usize,
) -> bool {
    unsafe { ReadProcessMemory(hproc, base_addr, lpbuf, size, None).is_ok() }
}

pub fn attach_console(pid: u32) -> bool {
    unsafe { AttachConsole(pid).is_ok() }
}

pub fn free_console() -> bool {
    unsafe { FreeConsole().is_ok() }
}

pub fn get_std_handle(stdhandle: STD_HANDLE) -> HANDLE {
    unsafe { GetStdHandle(stdhandle).unwrap_or_default() }
}

pub fn get_console_mode(h_console_handle: HANDLE) -> (CONSOLE_MODE, bool) {
    //log!("@get_console_mode: {}", h_console_handle.0 as isize);
    unsafe {
        let mut mode = CONSOLE_MODE::default();
        if GetConsoleMode(h_console_handle, &mut mode as *mut CONSOLE_MODE).is_ok() {
            (mode, true)
        } else {
            let err = get_last_error();
            log!("@ERR: {}", err.0);
            (mode, false)
        }
    }
}

pub fn get_console_process_list(pids: &mut [u32]) -> u32 {
    unsafe { GetConsoleProcessList(pids) }
}

pub fn set_std_handle(std_handle: STD_HANDLE, handle: HANDLE) -> bool {
    unsafe { SetStdHandle(std_handle, handle).is_ok() }
}

pub fn send_input(inputs: &[INPUT]) -> u32 {
    unsafe { SendInput(inputs, size_of::<INPUT>() as i32) }
}

pub fn get_console_screen_buffer_info(
    h_stdout: HANDLE,
    csbi: &mut CONSOLE_SCREEN_BUFFER_INFO,
) -> bool {
    unsafe { GetConsoleScreenBufferInfo(h_stdout, csbi as *mut CONSOLE_SCREEN_BUFFER_INFO).is_ok() }
}

pub fn read_console_output_character(
    h_stdout: HANDLE,
    wcbuf: &mut [u16],
    read_coord: COORD,
) -> u32 {
    let mut cch_read: u32 = 0;
    unsafe {
        _ = ReadConsoleOutputCharacterW(h_stdout, wcbuf, read_coord, &mut cch_read as *mut _);
    }
    cch_read
}

pub fn read_console_output_character_a(
    h_stdout: HANDLE,
    cbuf: &mut [u8],
    read_coord: COORD,
) -> u32 {
    let mut cch_read: u32 = 0;
    unsafe {
        let xx = ReadConsoleOutputCharacterA(h_stdout, cbuf, read_coord, &mut cch_read as *mut _);
        if cch_read == 0 {
            let err = get_last_error();
            log!("??ERR: {}", err.0);
        }
        match xx {
            Ok(_) => {}
            Err(e) => {
                log!("{:?}", e);
            }
        }
    }
    cch_read
}

pub fn get_file_attributes(file_path: &str) -> u32 {
    unsafe {
        let wsz_path = wsz_from_str(file_path);
        GetFileAttributesW(pwsz(&wsz_path))
    }
}

pub fn get_client_rect(hwnd: HWND, rect: &mut RECT) {
    unsafe {
        _ = GetClientRect(hwnd, rect as *mut RECT);
    }
}

pub fn get_environment_variable(name: &str, buf: &mut [u16]) -> u32 {
    let wsz_name = wsz_from_str(name);
    unsafe { GetEnvironmentVariableW(pwsz(&wsz_name), Some(buf)) }
}

pub fn create_file(
    path: &str,
    desired_access: u32,
    share_mode: FILE_SHARE_MODE,
    creation_disposition: FILE_CREATION_DISPOSITION,
    flags_and_attrs: FILE_FLAGS_AND_ATTRIBUTES,
) -> HANDLE {
    let wsz_path = wsz_from_str(path);
    unsafe {
        CreateFileW(
            pwsz(&wsz_path),
            desired_access,
            share_mode,
            None,
            creation_disposition,
            flags_and_attrs,
            None,
        )
        .unwrap_or_default()
    }
}

pub fn get_final_path_name_by_handle(
    hfile: HANDLE,
    buf: &mut [u16],
    flags: GETFINALPATHNAMEBYHANDLE_FLAGS,
) -> u32 {
    unsafe { GetFinalPathNameByHandleW(hfile, buf, flags) }
}

pub fn client_to_screen(hwnd: HWND, pt: &mut POINT) {
    unsafe {
        _ = ClientToScreen(hwnd, pt as *mut _);
    }
}

pub fn remove_prop(hwnd: HWND, name: &str) {
    let wsz_name = wsz_from_str(name);
    unsafe {
        _ = RemovePropW(hwnd, pwsz(&wsz_name));
    }
}

pub fn sleep(millis: u32) {
    unsafe {
        Sleep(millis);
    }
}

pub fn get_foreground_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

pub fn get_focus() -> HWND {
    unsafe { GetFocus() }
}

pub fn attach_thread_input(tid_attach: u32, tid_attach_to: u32, attach: BOOL) {
    unsafe {
        _ = AttachThreadInput(tid_attach, tid_attach_to, attach);
    }
}

pub fn get_current_thread_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}

pub fn get_local_time() -> SYSTEMTIME {
    unsafe { GetLocalTime() }
}

pub fn set_window_subclass(
    hwnd: HWND,
    subclass_proc: SUBCLASSPROC,
    idsubclass: usize,
    refdata: usize,
) -> bool {
    unsafe { SetWindowSubclass(hwnd, subclass_proc, idsubclass, refdata).0 != 0 }
}

pub fn create_font_indirect(lf: &LOGFONTW) -> HFONT {
    unsafe { CreateFontIndirectW(lf as _) }
}

pub fn get_dc(hwnd: HWND) -> HDC {
    unsafe { GetDC(hwnd) }
}

pub fn create_compatible_dc(hdc: HDC) -> HDC {
    unsafe { CreateCompatibleDC(hdc) }
}

pub fn release_dc(hwnd: HWND, hdc: HDC) {
    unsafe {
        ReleaseDC(hwnd, hdc);
    }
}

pub fn save_dc(hdc: HDC) -> i32 {
    unsafe { SaveDC(hdc) }
}

pub fn select_font(hdc: HDC, hfont: HFONT) -> HFONT {
    unsafe { HFONT(SelectObject(hdc, hfont).0) }
}

pub fn draw_text(hdc: HDC, text: &str, rc: &mut RECT, fmt: DRAW_TEXT_FORMAT) {
    let mut ws_text: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        DrawTextW(hdc, ws_text.as_mut_slice(), rc as _, fmt);
    }
}

pub fn draw_focus_rect(hdc: HDC, rc: &RECT) {
    unsafe {
        _ = DrawFocusRect(hdc, rc as _);
    }
}

pub fn get_sys_color_brush(color_index: SYS_COLOR_INDEX) -> HBRUSH {
    unsafe { GetSysColorBrush(color_index) }
}

pub fn get_sys_color(color_index: SYS_COLOR_INDEX) -> COLORREF {
    unsafe { COLORREF(GetSysColor(color_index)) }
}

pub fn fill_rect(hdc: HDC, rc: &RECT, hbr: HBRUSH) {
    unsafe {
        FillRect(hdc, rc as _, hbr);
    }
}

pub fn set_bk_color(hdc: HDC, color: COLORREF) {
    unsafe {
        SetBkColor(hdc, color);
    }
}

pub fn set_text_color(hdc: HDC, color: COLORREF) {
    unsafe {
        SetTextColor(hdc, color);
    }
}

pub fn get_theme_part_size(
    htheme: HTHEME,
    hdc: HDC,
    part_id: i32,
    state_id: i32,
    tsize: THEMESIZE,
) -> SIZE {
    unsafe { GetThemePartSize(htheme, hdc, part_id, state_id, None, tsize).unwrap_or_default() }
}

pub fn draw_theme_background(htheme: HTHEME, hdc: HDC, part_id: i32, state_id: i32, rc: &RECT) {
    unsafe {
        _ = DrawThemeBackground(htheme, hdc, part_id, state_id, rc as _, None);
    }
}

pub fn remove_window_subclass(hwnd: HWND, subclass_proc: SUBCLASSPROC, uidsubclass: usize) {
    unsafe {
        _ = RemoveWindowSubclass(hwnd, subclass_proc, uidsubclass);
    }
}

pub fn restore_dc(hdc: HDC, nsaveddc: i32) {
    unsafe {
        _ = RestoreDC(hdc, nsaveddc);
    }
}

pub fn delete_dc(hdc: HDC) {
    unsafe {
        _ = DeleteDC(hdc);
    }
}

pub fn delete_object(hobj: HGDIOBJ) {
    unsafe {
        _ = DeleteObject(hobj);
    }
}

pub fn close_theme_data(htheme: HTHEME) {
    unsafe {
        _ = CloseThemeData(htheme);
    }
}

pub fn def_subclass_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub fn get_window_rect(hwnd: HWND, rect: &mut RECT) {
    unsafe {
        _ = GetWindowRect(hwnd, rect as _);
    }
}

pub fn pt_in_rect(rect: &RECT, pt: POINT) -> bool {
    unsafe { PtInRect(rect as *const _, pt).as_bool() }
}

pub fn move_window(hwnd: HWND, x: i32, y: i32, w: i32, h: i32, repaint: bool) {
    unsafe {
        _ = MoveWindow(hwnd, x, y, w, h, repaint);
    }
}

pub fn set_timer(hwnd: HWND, id: usize, elapse: u32, timerfunc: TIMERPROC) {
    unsafe {
        _ = SetTimer(hwnd, id, elapse, timerfunc);
    }
}

pub fn get_parent(hwnd: HWND) -> HWND {
    unsafe { GetParent(hwnd).unwrap_or_default() }
}

pub fn begin_paint(hwnd: HWND, ps: &mut PAINTSTRUCT) {
    unsafe {
        BeginPaint(hwnd, ps as *mut _);
    }
}

pub fn end_paint(hwnd: HWND, ps: &PAINTSTRUCT) {
    unsafe {
        _ = EndPaint(hwnd, ps as *const _);
    }
}

pub fn get_cursor_pos(pt: &mut POINT) {
    unsafe {
        _ = GetCursorPos(pt as _);
    }
}

pub fn sh_get_folder_path(hwnd: Option<&HWND>, csidl: i32, htoken: HANDLE, flags: u32) -> String {
    const BUF_SIZE: usize = MAX_PATH as _;
    let mut buf = [0u16; BUF_SIZE];
    unsafe {
        if !SHGetFolderPathW(hwnd, csidl, htoken, flags, &mut buf).is_ok() {
            return String::new();
        }
    }
    let wsz_path = U16CString::from_vec_truncate(&buf);
    wsz_path.to_string_lossy()
}

pub fn create_directory(path: &str) -> bool {
    let wsz_path = wsz_from_str(path);
    unsafe { CreateDirectoryW(pwsz(&wsz_path), None).is_ok() }
}

pub fn open_theme_data(hwnd: HWND, classlist: &str) -> HTHEME {
    let wsz_classlist = wsz_from_str(classlist);
    unsafe { OpenThemeData(hwnd, pwsz(&wsz_classlist)) }
}

pub fn listbox_getcursel(hwnd: HWND) -> i32 {
    unsafe { SendMessageW(hwnd, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as _ }
}

pub fn listbox_setcursel(hwnd: HWND, sel: i32) {
    unsafe {
        SendMessageW(hwnd, LB_SETCURSEL, WPARAM(sel as _), LPARAM(0));
    }
}

pub fn screen_to_client(hwnd: HWND, pt: &mut POINT) {
    unsafe {
        _ = ScreenToClient(hwnd, pt as *mut _);
    }
}

pub fn is_debugger_present() -> bool {
    unsafe { IsDebuggerPresent().as_bool() }
}

pub fn get_current_process_id() -> u32 {
    unsafe { GetCurrentProcessId() }
}

pub fn get_tick_count() -> u32 {
    unsafe { GetTickCount() }
}

pub fn get_tick_count64() -> u64 {
    unsafe { GetTickCount64() }
}

pub fn kill_timer(hwnd: HWND, id: usize) {
    unsafe {
        _ = KillTimer(hwnd, id);
    }
}

pub fn get_desktop_window() -> HWND {
    unsafe { GetDesktopWindow() }
}

pub fn get_text_metrics(hdc: HDC, tm: &mut TEXTMETRICW) -> bool {
    unsafe { GetTextMetricsW(hdc, tm as *mut TEXTMETRICW).into() }
}

pub fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

pub fn get_text_extend_point(hdc: HDC, str: &str, sz: &mut SIZE) -> bool {
    let wsz = wsz_from_str(str);
    unsafe { GetTextExtentPoint32W(hdc, wsz.as_slice(), sz).as_bool() }
}

pub fn get_system_metrics(index: SYSTEM_METRICS_INDEX) -> i32 {
    unsafe { GetSystemMetrics(index) }
}

pub fn set_scroll_info(hwnd: HWND, bar: SCROLLBAR_CONSTANTS, si: &SCROLLINFO, redraw: bool) -> i32 {
    unsafe { SetScrollInfo(hwnd, bar, si as _, redraw) }
}

pub fn set_window_pos(
    hwnd: HWND,
    hwnd_after: HWND,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    flags: SET_WINDOW_POS_FLAGS,
) -> bool {
    unsafe { SetWindowPos(hwnd, hwnd_after, x, y, cx, cy, flags).is_ok() }
}

pub fn invalidate_rect(hwnd: HWND, rect: Option<&RECT>, erase: bool) {
    unsafe {
        _ = InvalidateRect(hwnd, rect.map(|x| x as _), erase);
    }
}

pub fn update_window(hwnd: HWND) {
    unsafe {
        _ = UpdateWindow(hwnd);
    }
}

pub fn create_compatible_bitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP {
    unsafe { CreateCompatibleBitmap(hdc, cx, cy) }
}

pub fn select_object(hdc: HDC, hobj: HGDIOBJ) -> HGDIOBJ {
    unsafe { SelectObject(hdc, hobj) }
}

pub fn set_bk_mode(hdc: HDC, mode: BACKGROUND_MODE) -> i32 {
    unsafe { SetBkMode(hdc, mode) }
}

pub fn get_scroll_info(hwnd: HWND, bar: SCROLLBAR_CONSTANTS, si: &mut SCROLLINFO) -> bool {
    unsafe { GetScrollInfo(hwnd, bar, si).is_ok() }
}

pub fn bit_blt(
    hdc: HDC,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    hdc_src: HDC,
    x1: i32,
    y1: i32,
    rop: ROP_CODE,
) -> bool {
    unsafe { BitBlt(hdc, x, y, cx, cy, hdc_src, x1, y1, rop).is_ok() }
}

pub fn set_scroll_pos(hwnd: HWND, bar: SCROLLBAR_CONSTANTS, pos: i32, redraw: bool) -> i32 {
    unsafe { SetScrollPos(hwnd, bar, pos, redraw) }
}

pub fn find_first_file_ex(
    file_name: &str,
    info_level: FINDEX_INFO_LEVELS,
    find_data: &mut WIN32_FIND_DATAW,
    search_ops: FINDEX_SEARCH_OPS,
    additional_flags: FIND_FIRST_EX_FLAGS,
) -> HANDLE {
    let wsz_file_name = wsz_from_str(file_name);
    unsafe {
        FindFirstFileExW(
            pwsz(&wsz_file_name),
            info_level,
            find_data as *mut WIN32_FIND_DATAW as _,
            search_ops,
            None,
            additional_flags,
        )
        .unwrap_or_default()
    }
}

pub fn find_next_file(hfind: HANDLE, find_data: &mut WIN32_FIND_DATAW) -> bool {
    unsafe { FindNextFileW(hfind, find_data as *mut WIN32_FIND_DATAW as _).is_ok() }
}

pub fn get_current_console_font_ex(
    hstdout: HANDLE,
    maximium_window: bool,
    console_font_info_ex: &mut CONSOLE_FONT_INFOEX,
) -> bool {
    unsafe {
        GetCurrentConsoleFontEx(hstdout, maximium_window, console_font_info_ex as *mut _).is_ok()
    }
}

pub fn set_console_cursor_position(hstdout: HANDLE, cur_pos: COORD) -> bool {
    unsafe { SetConsoleCursorPosition(hstdout, cur_pos).is_ok() }
}

pub fn write_console_input(hstdin: HANDLE, inputs: &[INPUT_RECORD], num_written: &mut u32) -> bool {
    unsafe { WriteConsoleInputW(hstdin, inputs, num_written as _).is_ok() }
}

pub fn multi_byte_to_wide_char(
    code_page: u32,
    flags: MULTI_BYTE_TO_WIDE_CHAR_FLAGS,
    mb_str: &[u8],
    w_str: Option<&mut [u16]>,
) -> i32 {
    unsafe { MultiByteToWideChar(code_page, flags, mb_str, w_str) }
}

pub fn wide_char_to_multi_byte(
    code_page: u32,
    flags: u32,
    wc_str: &[u16],
    mb_str: Option<&mut [u8]>,
) -> i32 {
    unsafe { WideCharToMultiByte(code_page, flags, wc_str, mb_str, PCSTR::null(), None) }
}

pub fn message_beep(typ: MESSAGEBOX_STYLE) -> bool {
    unsafe { MessageBeep(typ).is_ok() }
}

pub fn offset_rect(rc: &mut RECT, dx: i32, dy: i32) -> bool {
    unsafe { OffsetRect(rc, dx, dy).as_bool() }
}

pub(crate) fn write_private_profile_string(
    section: &str,
    key: &str,
    value: &str,
    file_name: &str,
) -> bool {
    let wsz_section = wsz_from_str(section);
    let wsz_key = wsz_from_str(key);
    let wsz_value = wsz_from_str(&value);
    let wsz_file_name = wsz_from_str(&file_name);
    unsafe {
        WritePrivateProfileStringW(
            pwsz(&wsz_section),
            pwsz(&wsz_key),
            pwsz(&wsz_value),
            pwsz(&wsz_file_name),
        )
        .is_ok()
    }
}

pub fn get_private_profile_string(
    section: &str,
    key: &str,
    def_val: Option<&str>,
    file_name: &str,
) -> String {
    let wsz_section = wsz_from_str(section);
    let wsz_key = wsz_from_str(key);
    let wsz_file_name = wsz_from_str(&file_name);

    let wsz_def_val: U16CString;
    let pwsz_def_val = if let Some(def_val) = def_val {
        wsz_def_val = wsz_from_str(def_val);
        pwsz(&wsz_def_val)
    } else {
        PCWSTR::null()
    };

    let mut buf = [0u16; 1024];
    let cch = unsafe {
        GetPrivateProfileStringW(
            pwsz(&wsz_section),
            pwsz(&wsz_key),
            pwsz_def_val,
            Some(&mut buf),
            pwsz(&wsz_file_name),
        )
    };
    String::from_utf16_lossy(&buf[..cch as usize])
}

pub fn get_dlg_item(hwnd_dlg: HWND, item_id: u16) -> HWND {
    unsafe { GetDlgItem(hwnd_dlg, item_id as _).unwrap_or_default() }
}

pub fn set_window_text(hwnd: HWND, text: &str) -> bool {
    let wsz_text = wsz_from_str(text);
    unsafe { SetWindowTextW(hwnd, pwsz(&wsz_text)).is_ok() }
}

pub fn sh_get_known_folder_path(fid: &GUID, flags: KNOWN_FOLDER_FLAG) -> String {
    let wsz_empty = PWSTR::from_raw(&mut [0u16] as _);
    unsafe {
        let pwsz = SHGetKnownFolderPath(fid as _, flags, None).unwrap_or(wsz_empty);
        U16CStr::from_ptr_str_mut(pwsz.0).to_string_lossy()
    }
}

pub fn co_initialize() -> bool {
    unsafe { CoInitialize(None).is_ok() }
}

pub fn co_create_instance<T: Interface>(clsid: &GUID) -> Result<T> {
    unsafe { CoCreateInstance(clsid as _, None, CLSCTX_INPROC_SERVER) }
}

pub fn shell_execute(
    hwnd: HWND,
    operation: &str,
    file: &str,
    params: Option<&str>,
    dir: Option<&str>,
    show_cmd: SHOW_WINDOW_CMD,
) -> HINSTANCE {
    let wsz_operation = wsz_from_str(operation);
    let wsz_file = wsz_from_str(file);

    let mut pwsz_params = PCWSTR::null();
    let wsz_params: U16CString;
    if let Some(params) = params {
        wsz_params = wsz_from_str(params);
        pwsz_params = PCWSTR(wsz_params.as_ptr());
    }

    let mut pwsz_dir = PCWSTR::null();
    let wsz_dir: U16CString;
    if let Some(dir) = dir {
        wsz_dir = wsz_from_str(dir);
        pwsz_dir = PCWSTR(wsz_dir.as_ptr());
    }

    unsafe {
        ShellExecuteW(
            hwnd,
            pwsz(&wsz_operation),
            pwsz(&wsz_file),
            pwsz_params,
            pwsz_dir,
            show_cmd,
        )
    }
}

pub fn message_box(
    hwnd: HWND,
    text: &str,
    caption: &str,
    typ: MESSAGEBOX_STYLE,
) -> MESSAGEBOX_RESULT {
    let wsz_text = wsz_from_str(text);
    let wsz_caption = wsz_from_str(caption);
    unsafe { MessageBoxW(hwnd, pwsz(&wsz_text), pwsz(&wsz_caption), typ) }
}

pub fn beep(freq: u32, dur: u32) {
    _ = unsafe { Beep(freq, dur) };
}

pub fn write_console_output_character(hstdout: HANDLE, text: &str, coord: COORD) -> u32 {
    let w_text: Vec<u16> = text.encode_utf16().collect();
    let mut cch_written = 0u32;
    _ = unsafe { WriteConsoleOutputCharacterW(hstdout, &w_text, coord, &mut cch_written as _) };
    cch_written
}

pub fn write_console_output_character0(hstdout: HANDLE, wcs: &[u16], coord: COORD) -> u32 {
    let mut cch_written = 0u32;
    _ = unsafe { WriteConsoleOutputCharacterW(hstdout, wcs, coord, &mut cch_written as _) };
    cch_written
}

pub fn write_console(hstdout: HANDLE, text: &str) -> u32 {
    let w_text: Vec<u16> = text.encode_utf16().collect();
    let mut cch_written = 0u32;
    _ = unsafe { WriteConsoleW(hstdout, &w_text, Some(&mut cch_written as _), None) };
    cch_written
}

pub fn get_command_line() -> String {
    unsafe { GetCommandLineW().to_string().unwrap() }
}

pub fn alloc_console() -> bool {
    unsafe { AllocConsole().is_ok() }
}

pub fn exit_process(code: u32) {
    unsafe {
        ExitProcess(code);
    }
}

pub fn read_console_input(hstdin: HANDLE, buf: &mut [INPUT_RECORD], read_count: &mut u32) -> bool {
    unsafe { ReadConsoleInputW(hstdin, buf, read_count as _).is_ok() }
}

pub fn wait_for_single_object(handle: HANDLE, millis: u32) -> WAIT_EVENT {
    unsafe { WaitForSingleObject(handle, millis) }
}

pub fn set_console_title(title: &str) -> bool {
    let wsz_title = wsz_from_str(title);
    unsafe { SetConsoleTitleW(pwsz(&wsz_title)).is_ok() }
}

pub fn set_console_ctrl_handler(handler: PHANDLER_ROUTINE, add: bool) -> bool {
    unsafe { SetConsoleCtrlHandler(handler, add).is_ok() }
}

pub fn vk_key_scan(wc: u16) -> i16 {
    unsafe { VkKeyScanW(wc) }
}
pub fn get_console_title() -> String {
    let mut buf = [0u16; 128];
    let cch = unsafe { GetConsoleTitleW(&mut buf) };
    String::from_utf16_lossy(&buf[..cch as usize])
}

pub fn safe_array_destroy(sa: *const SAFEARRAY) {
    unsafe {
        _ = SafeArrayDestroy(sa);
    }
}

pub fn safe_array_lock(sa: *const SAFEARRAY) -> bool {
    unsafe { SafeArrayLock(sa).is_ok() }
}

pub fn safe_array_unlock(sa: *const SAFEARRAY) -> bool {
    unsafe { SafeArrayUnlock(sa).is_ok() }
}

pub fn post_thread_message(thread_id: u32, msg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
    unsafe { PostThreadMessageW(thread_id, msg, wparam, lparam).is_ok() }
}

pub fn create_thread(
    thread_proc: LPTHREAD_START_ROUTINE,
    params: Option<*const c_void>,
) -> (HANDLE, u32) {
    unsafe {
        let mut tid = 0u32;
        let hthread = CreateThread(
            None,
            0,
            thread_proc,
            params,
            THREAD_CREATION_FLAGS(0),
            Some(&mut tid),
        )
        .unwrap_or_default();
        (hthread, tid)
    }
}

pub fn co_initialize_ex(coinit: COINIT) -> bool {
    unsafe { CoInitializeEx(None, coinit).is_ok() }
}

pub fn get_message(msg: &mut MSG, hwnd: HWND) -> bool {
    unsafe { GetMessageW(msg as _, hwnd, 0, 0).as_bool() }
}

pub fn create_event(manual_reset: bool, initial_state: bool) -> HANDLE {
    unsafe { CreateEventW(None, manual_reset, initial_state, None).unwrap_or_default() }
}

pub fn set_event(hevent: HANDLE) -> bool {
    unsafe { SetEvent(hevent).is_ok() }
}

pub fn reset_event(hevent: HANDLE) -> bool {
    unsafe { ResetEvent(hevent).is_ok() }
}

pub fn msg_wait_for_multiple_objects(
    handles: &[HANDLE],
    wait_all: bool,
    millis: u32,
    wake_mask: QUEUE_STATUS_FLAGS,
) -> WAIT_EVENT {
    unsafe { MsgWaitForMultipleObjects(Some(handles), wait_all, millis, wake_mask) }
}

pub fn open_thread(access: THREAD_ACCESS_RIGHTS, inherit: bool, tid: u32) -> HANDLE {
    unsafe { OpenThread(access, inherit, tid).unwrap_or_default() }
}

pub fn co_uninitialize() {
    unsafe { CoUninitialize() }
}

pub fn find_close(hfind: HANDLE) {
    unsafe { _ = FindClose(hfind) };
}

pub fn get_window_long(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> i32 {
    unsafe { GetWindowLongW(hwnd, index) }
}

pub fn set_window_long(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: i32) {
    unsafe {
        SetWindowLongW(hwnd, index, value);
    }
}

pub fn create_solid_brush(color: COLORREF) -> HBRUSH {
    unsafe { CreateSolidBrush(color) }
}

pub fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    let value: u32 = r as u32 | ((g as u16) << 8) as u32 | (b as u32) << 16;
    COLORREF(value)
}

pub fn get_window(hwnd: HWND, cmd: GET_WINDOW_CMD) -> HWND {
    unsafe { GetWindow(hwnd, cmd).unwrap_or_default() }
}

pub fn set_parent(hwnd_child: HWND, hwnd_new_parent: HWND) {
    unsafe {
        _ = SetParent(hwnd_child, hwnd_new_parent);
    }
}

pub fn set_win_event_hook(
    event_min: u32,
    event_max: u32,
    hmod: HMODULE,
    proc: WINEVENTPROC,
    pid: u32,
    tid: u32,
    flags: u32,
) -> HWINEVENTHOOK {
    unsafe { SetWinEventHook(event_min, event_max, hmod, proc, pid, tid, flags) }
}

pub fn unhook_win_event(h_win_event_hook: HWINEVENTHOOK) -> bool {
    unsafe { UnhookWinEvent(h_win_event_hook).as_bool() }
}

pub fn ext_text_out(
    hdc: HDC,
    x: i32,
    y: i32,
    options: ETO_OPTIONS,
    rect: Option<&RECT>,
    text: String,
) -> bool {
    let wsz_text = wsz_from_str(&text);
    unsafe {
        ExtTextOutW(
            hdc,
            x,
            y,
            options,
            rect.map(|x| x as _),
            pwsz(&wsz_text),
            wsz_text.len() as u32,
            None,
        )
        .as_bool()
    }
}

pub fn set_text_align(hdc: HDC, align: TEXT_ALIGN_OPTIONS) -> TEXT_ALIGN_OPTIONS {
    unsafe { TEXT_ALIGN_OPTIONS(SetTextAlign(hdc, align)) }
}

pub fn set_process_dpi_awareness_context(context: DPI_AWARENESS_CONTEXT) -> bool {
    unsafe { SetProcessDpiAwarenessContext(context).is_ok() }
}

pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
    unsafe { GetDpiForWindow(hwnd) }
}

pub fn mul_div(number: i32, numerator: i32, denominator: i32) -> i32 {
    unsafe { MulDiv(number, numerator, denominator) }
}

pub fn get_temp_path(buf: Option<&mut [u16]>) -> u32 {
    unsafe { GetTempPathW(buf) }
}

pub fn fill_console_output_character(
    h_console_output: HANDLE,
    c: u16,
    count: u32,
    coord: COORD,
    cch_write: &mut u32,
) -> bool {
    unsafe {
        FillConsoleOutputCharacterW(h_console_output, c, count, coord, cch_write as _).is_ok()
    }
}

pub fn lock_window_update(hwnd: HWND) -> bool {
    unsafe { LockWindowUpdate(hwnd).as_bool() }
}

pub fn flush_console_input_buffer(hstdin: HANDLE) -> bool {
    unsafe { FlushConsoleInputBuffer(hstdin).is_ok() }
}

pub fn generate_console_ctrl_event(ctrl_event: u32, process_group_id: u32) -> bool {
    unsafe { GenerateConsoleCtrlEvent(ctrl_event, process_group_id).is_ok() }
}
