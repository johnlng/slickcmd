use crate::startup_link::StartupLink;
use slickcmd_common::consts::{IDC_CHK_CALCULATOR, IDC_CHK_CD_COMPLETION, IDC_CHK_RUN_ON_STARTUP, IDC_CHK_SHOW_CLOCK, IDC_MAX_RECENT_DIRS, IDC_MAX_RECENT_DIRS_SPIN, IDD_OPTIONS};
use slickcmd_common::dlg::{dlg_proc, Dlg};
use slickcmd_common::{dlg, utils, win32};
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::global::GLOBAL;

#[derive(Default)]
pub struct OptionsDlg {
    hwnd: HWND,

    hwnd_max_recent_dirs: HWND,
    hwnd_chk_cd_completion: HWND,
    hwnd_chk_run_on_startup: HWND,
    hwnd_chk_show_clock: HWND,
    hwnd_chk_direct_calculator: HWND,
}

impl OptionsDlg {
    pub fn new() -> OptionsDlg {
        OptionsDlg::default()
    }

    pub fn show(&mut self) {
        dlg::set_creating_dlg(self);

        win32::dialog_box(
            GLOBAL.hinstance(),
            IDD_OPTIONS,
            GLOBAL.hwnd_main(),
            Some(dlg_proc::<Self>),
        );
    }

    fn on_init_dialog(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> isize {

        self.hwnd_max_recent_dirs = win32::get_dlg_item(self.hwnd, IDC_MAX_RECENT_DIRS);
        let hwnd_spin = win32::get_dlg_item(self.hwnd, IDC_MAX_RECENT_DIRS_SPIN);
        win32::send_message(
            hwnd_spin,
            UDM_SETRANGE,
            WPARAM(0),
            utils::make_lparam(35, 1),
        );

        self.hwnd_chk_cd_completion = win32::get_dlg_item(self.hwnd, IDC_CHK_CD_COMPLETION);
        self.hwnd_chk_run_on_startup = win32::get_dlg_item(self.hwnd, IDC_CHK_RUN_ON_STARTUP);
        self.hwnd_chk_show_clock = win32::get_dlg_item(self.hwnd, IDC_CHK_SHOW_CLOCK);
        self.hwnd_chk_direct_calculator = win32::get_dlg_item(self.hwnd, IDC_CHK_CALCULATOR);

        let options = &GLOBAL.options;
        let text = &format!("{}", options.max_recent_dirs());
        win32::set_window_text(self.hwnd_max_recent_dirs, text);

        self.set_check(self.hwnd_chk_cd_completion, options.cd_completion());
        self.set_check(self.hwnd_chk_run_on_startup, options.run_on_startup());
        self.set_check(self.hwnd_chk_show_clock, options.show_clock());
        self.set_check(self.hwnd_chk_direct_calculator, options.direct_calculator());

        1
    }

    fn set_check(&self, hwnd: HWND, checked: bool) {
        let wparam = if checked { WPARAM(1) } else { WPARAM(0) };
        win32::send_message(hwnd, BM_SETCHECK, wparam, LPARAM(0));
    }

    fn get_check(&self, hwnd: HWND) -> bool {
        win32::send_message(hwnd, BM_GETCHECK, WPARAM(0), LPARAM(0)) == LRESULT(BST_CHECKED.0 as _)
    }

    fn on_ok(&self) {
        let text = win32::get_window_text(self.hwnd_max_recent_dirs);
        let max_recent_dirs = text.parse::<u32>().unwrap_or_default();
        if max_recent_dirs < 1 || max_recent_dirs > 35 {
            utils::alert("Max recent dirs out of range (1 to 35)");
            return;
        }

        let enable_cd_completion = self.get_check(self.hwnd_chk_cd_completion);
        let run_on_startup = self.get_check(self.hwnd_chk_run_on_startup);
        let show_clock = self.get_check(self.hwnd_chk_show_clock);
        let direct_calculator = self.get_check(self.hwnd_chk_direct_calculator);

        let options = &GLOBAL.options;
        options.set_max_recent_dirs(max_recent_dirs);
        options.set_cd_completion(enable_cd_completion);
        options.set_run_on_startup(run_on_startup);
        options.set_show_clock(show_clock);
        options.set_direct_calculator(direct_calculator);
        options.save();

        //
        let link_exists = StartupLink::exists().unwrap_or_default();
        if options.run_on_startup() {
            if !link_exists && StartupLink::create().is_err() {
                utils::alert("Failed to create Startup link")
            }
        } else {
            if link_exists && !StartupLink::remove() {
                utils::alert("Failed to remove Startup link")
            }
        }

        win32::end_dialog(self.hwnd, IDOK);
    }
}

impl Dlg for OptionsDlg {
    fn set_hwnd(&mut self, hwnd: HWND) {
        self.hwnd = hwnd;
    }

    fn dlg_proc(&mut self, _window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> isize {
        match message {
            WM_INITDIALOG => {
                return self.on_init_dialog(wparam, lparam);
            }
            WM_COMMAND => {
                let id = (wparam.0 as i32) & 0xffff;
                if id == IDOK.0 {
                    self.on_ok();
                } else if id == IDCANCEL.0 {
                    win32::end_dialog(self.hwnd, IDCANCEL);
                }
                return 1;
            }
            _ => {}
        }
        0
    }
}
