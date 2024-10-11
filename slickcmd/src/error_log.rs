use log::{error, LevelFilter};
use simplelog::{CombinedLogger, Config, WriteLogger};
use slickcmd_common::{utils, win32};
use std::fs::{OpenOptions};
use std::panic;

pub fn init() {
    let mut path = utils::get_appdata_local_dir();
    path.push_str("\\slickcmd");
    win32::create_directory(&path);
    path.push('\\');
    path.push_str("error.log");

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )])
    .unwrap();

    panic::set_hook(Box::new(move |info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        match info.location() {
            Some(location) => {
                error!(
                    target: "panic", "panicked at '{}': {}:{}",
                    msg,
                    location.file(),
                    location.line()
                );
            }
            None => error!(
                target: "panic",
                "panicked at '{}'",
                msg
            ),
        }
    }));
}
