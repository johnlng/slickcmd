pub const IDD_SLICKCMD_DIALOG: u16 = 102;
pub const IDD_ABOUTBOX: u16 = 103;
pub const IDM_OPTIONS: u16 = 104;
pub const IDM_ABOUT: u16 = 105;
pub const IDM_EXIT: u16 = 106;
pub const IDI_SLICKCMD: u16 = 107;
pub const IDI_SMALL: u16 = 108;
pub const IDC_SLICKCMD: u16 = 109;
pub const IDM_MANUAL: u16 = 110;

pub const IDD_OPTIONS: u16 = 131;
pub const IDC_MAX_RECENT_DIRS: u16 = 1002;
pub const IDC_MAX_RECENT_DIRS_SPIN: u16 = 1003;
pub const IDC_CHK_CD_COMPLETION: u16 = 1005;
pub const IDC_CHK_RUN_ON_STARTUP: u16 = 1006;
pub const IDC_SYSLINK_SITE: u16 = 1007;
pub const IDC_CHK_SHOW_CLOCK: u16 = 1008;
pub const IDC_CHK_CALCULATOR: u16 = 1009;

//
pub const WM_USER: u32 = 0x0400;

pub const NIN_SELECT: u32 = WM_USER;
pub const NIN_KEYSELECT: u32 = NIN_SELECT|0x1;

//
pub const WM_CORE_SUPPRESS_INPUT_EVENT: u32 = WM_USER + 1001;
pub const WM_CORE_KEYDOWN: u32 = WM_USER + 1002;
pub const WM_CORE_KEYUP: u32 = WM_USER + 1003;
pub const WM_NOTIFY_AC_LIST_CLOSED: u32 = WM_USER + 1004;

// pub const WM_POST_SHOW_MENU: u32 = WM_USER + 1005;
pub const WM_SHOW_MENU: u32 = WM_USER + 1005;
pub const WM_NOTIFY_KEY_SUPPRESS_END: u32 = WM_USER + 1006;
// pub const WM_SET_ENV_VAR: u32 = WM_USER + 1007;

//slickcmd
pub const WM_POST_CONSOLE_ACTIVATE: u32 = WM_USER + 4001;
pub const WM_TRAY_CALLBACK: u32 = WM_USER + 4002;

pub const WM_POST_ACTION: u32 = WM_USER + 4003;
pub const POST_ACTION_ALT_DOWN: usize = 1;
pub const POST_ACTION_CUSTOM_OUTPUT: usize = 2;

pub const WM_SHOW_AUTO_COMPLETE: u32 = WM_USER + 4004;
pub const WM_HIDE_AUTO_COMPLETE: u32 = WM_USER + 4005;
pub const WM_MOUSEDOWN_SHOWING_ACL: u32 = WM_USER + 4006;

pub const WM_SHOW_MENU_RESULT: u32 = WM_USER + 4007;
pub const WM_HIST_WIN_DESTROYED: u32 = WM_USER + 4008;

pub const WM_UIA_FOCUS_CHANGE: u32 = WM_USER + 4009;
pub const WM_WT_CONSOLE_ACTIVATE: u32 = WM_USER + 4010;
pub const WM_CLEAN_CONSOLES: u32 = WM_USER + 4011;

pub const WM_SYSTEM_MOVESIZESTART: u32 = WM_USER + 4012;
pub const WM_SYSTEM_MOVESIZEEND: u32 = WM_USER + 4013;
pub const WM_WT_FOCUS_CHANGE: u32 = WM_USER + 4014;

pub const WM_POST_CREATE: u32 = WM_USER + 4015;

//
pub const APP_TITLE: &str = "Slick Cmd";
