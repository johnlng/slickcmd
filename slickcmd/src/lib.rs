use crate::global::Global;

pub mod app;
pub mod command_hist;
pub mod command_hist_list;
pub mod command_hist_win;
pub mod console;
pub mod dir_complete;
pub mod dir_man;
pub mod global;
pub mod key_hook_suppressor;
pub mod keyboard_input;
pub mod main_win;
pub mod msg_win;
pub mod options;
pub mod options_dlg;
pub mod shell;
pub mod startup_link;
pub mod tray_icon;
pub mod error_log;
pub mod manual;

static GLOBAL: Global = Global::new();
