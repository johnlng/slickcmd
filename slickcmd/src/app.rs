use crate::app_state::AppState;
use crate::console_man::ConsoleMan;
use crate::global::GLOBAL;
use crate::main_win::MainWin;
use crate::msg_win::MsgWin;
use crate::options::Options;
use crate::startup_link::StartupLink;
use crate::tray_icon::TrayIcon;
use crate::win_man::WinMan;
use slickcmd_common::consts::{APP_TITLE, IDI_SMALL, WM_TRAY_CALLBACK};
use slickcmd_common::ini::Ini;
use slickcmd_common::{consts, utils, win32, winproc};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use windows::Win32::Foundation::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct App {
    main_win: MainWin,

    msg_win: MsgWin,

    tray_icon: TrayIcon,

    hhook_shell: HHOOK,

    state: Rc<AppState>,

}

unsafe impl Sync for App {}
unsafe impl Send for App {}

impl App {

    pub fn new() -> App {
        let state = Rc::new(AppState::default());

        let console_man = ConsoleMan::new(state.clone());
        let console_man = Rc::new(RefCell::new(console_man));

        let win_man = WinMan::new(console_man.clone());
        let win_man = Rc::new(RefCell::new(win_man));

        let msg_win = MsgWin::new(win_man.clone(), console_man.clone());
        App {
            main_win: MainWin::default(),
            msg_win,
            tray_icon: TrayIcon::default(),
            hhook_shell: HHOOK::default(),
            state,
        }
    }

    pub fn init(&mut self) -> bool {

        win32::set_process_dpi_awareness_context(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let _ = win32::create_mutex(false, "slck_cmd_mutex");
        if win32::get_last_error() == ERROR_ALREADY_EXISTS {
            return false;
        }
        win32::co_initialize();

        GLOBAL.init();
        let hinstance = GLOBAL.hinstance();

        let hwnd_main = self.main_win.create();
        GLOBAL.set_hwnd_main(hwnd_main);
        GLOBAL.set_dpi(win32::get_dpi_for_window(hwnd_main));

        self.state.recent_dirs.load();

        let hwnd_msg = self.msg_win.create();
        GLOBAL.set_hwnd_msg(hwnd_msg);

        let hicon = win32::load_icon(hinstance, IDI_SMALL);

        let exe_path = env::current_exe().unwrap().into_os_string();
        let md5 = md5::compute(exe_path.as_encoded_bytes());
        let guid = utils::u8s_as_guid(&md5.0);
        _ = self.tray_icon.create(hicon, APP_TITLE, hwnd_main, WM_TRAY_CALLBACK, &guid, 0);

        if !self.init_shell_hook() {
            return false;
        }

        let ini = Ini::new(Some(&Options::get_ini_path()));
        if ini.read::<String>("General", "run_on_startup") == "" {
            self.ask_if_run_on_startup();
        }

        StartupLink::sync_state(GLOBAL.options.run_on_startup());

        true
    }

    fn ask_if_run_on_startup(&self) {
        let msg = "Do you want Slick Cmd to run on startup?";
        let result = win32::message_box(
            GLOBAL.hwnd_main(),
            msg,
            APP_TITLE,
            MB_YESNO | MB_ICONQUESTION,
        );
        let options = &GLOBAL.options;
        options.set_run_on_startup(result == IDYES);
        options.save();
    }

    pub fn run(&mut self) {
        let hinstance = GLOBAL.hinstance();
        let haccel: HACCEL = win32::load_accelerators(hinstance, consts::IDC_SLICKCMD);
        winproc::message_loop(haccel);
    }

    pub fn finalize(&mut self) {
        self.state.recent_dirs.save();
        win32::unhook_widows_hook_ex(self.hhook_shell);
        self.tray_icon.destroy();
        self.msg_win.destroy();
    }

    fn init_shell_hook(&mut self) -> bool {
        let homd_shell = win32::load_library("slickcmd_shl.dll").unwrap_or_default();
        if homd_shell.is_invalid() {
            return false;
        }
        let p_shell_proc = win32::get_proc_address(homd_shell, "ShlProc");
        let shell_proc: HOOKPROC = Some(unsafe { std::mem::transmute(p_shell_proc) });
        let hhook = win32::set_windows_hook_ex(WH_SHELL, shell_proc, homd_shell.into(), 0);
        if hhook.is_invalid() {
            return false;
        }
        self.hhook_shell = hhook;
        true
    }

    //utils

    pub fn dpi_aware_value(value: i32) -> i32 {
        win32::mul_div(value, GLOBAL.dpi() as _, USER_DEFAULT_SCREEN_DPI as _)
    }

}
