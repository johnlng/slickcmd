#![windows_subsystem = "windows"]

use std::process::ExitCode;

use anyhow::Result;
use slickcmd::app::App;
use slickcmd::{error_log, manual};
use slickcmd_common::{logger, win32};

fn main() -> Result<ExitCode> {

    error_log::init();

    let cmdline = win32::get_command_line();
    if cmdline.ends_with("--man") {
        manual::show();
        return Ok(ExitCode::SUCCESS);
    }

    let mut app = App::default();

    if !app.init() {
        return Ok(ExitCode::FAILURE);
    }

    logger::cls();

    app.run();

    app.finalize();

    Ok(ExitCode::SUCCESS)
}
