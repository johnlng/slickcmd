use slickcmd_common::ini::Ini;
use slickcmd_common::utils;
use std::cell::Cell;

#[derive(Default)]
pub struct Options {

    max_recent_dirs: Cell<u32>,
    cd_completion: Cell<bool>,
    run_on_startup: Cell<bool>,
    show_clock: Cell<bool>,
}

impl Options {

    pub fn get_ini_path() -> String {
        let path = utils::get_appdata_local_dir();
        path + "\\slickcmd\\slickcmd.ini"
    }

    pub fn save(&self) {
        let ini = Ini::new(Some(&Self::get_ini_path()));
        ini.write("General", "max_recent_dirs", self.max_recent_dirs());
        ini.write("General", "cd_completion", self.cd_completion());
        ini.write("General", "run_on_startup", self.run_on_startup());
        ini.write("General", "show_clock", self.show_clock());
    }

    pub fn init(&self) {
        let ini = Ini::new(Some(&Self::get_ini_path()));
        self.set_max_recent_dirs(ini.read_or("General", "max_recent_dirs", 15));
        self.set_cd_completion(ini.read_or("General", "cd_completion", true));
        self.set_run_on_startup(ini.read_or("General", "run_on_startup", true));
        self.set_show_clock(ini.read_or("General", "show_clock", false));
    }

    pub fn max_recent_dirs(&self) -> u32 {
        self.max_recent_dirs.get()
    }

    pub fn set_max_recent_dirs(&self, value: u32) {
        self.max_recent_dirs.set(value);
    }

    pub fn cd_completion(&self) -> bool {
        self.cd_completion.get()
    }

    pub fn set_cd_completion(&self, value: bool) {
        self.cd_completion.set(value);
    }

    pub fn run_on_startup(&self) -> bool {
        self.run_on_startup.get()
    }

    pub fn set_run_on_startup(&self, value: bool) {
        self.run_on_startup.set(value);
    }

    pub fn show_clock(&self) -> bool {
        self.show_clock.get()
    }

    pub fn set_show_clock(&self, value: bool) {
        self.show_clock.set(value);
    }
}