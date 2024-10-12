use slickcmd_common::{utils, win32};
use std::fmt::Write;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::{fs, path::Path};

#[derive(Clone)]
pub struct CommandInfo {
    pub time: u64,
    pub command: String,
}

impl CommandInfo {
    pub fn new(command: &str) -> CommandInfo {
        let st = win32::get_local_time();

        let time: u64 = st.wYear as u64 * 100_00_00_00_00_000
            + st.wMonth as u64 * 100_00_00_00_000
            + st.wDay as u64 * 100_00_00_000
            + st.wHour as u64 * 100_00_000
            + st.wMinute as u64 * 100_000
            + st.wSecond as u64 * 1000
            + st.wMilliseconds as u64;

        CommandInfo {
            command: command.into(),
            time: time,
        }
    }
}

#[derive(Default, Clone)]
pub struct CommandHist {
    pub category: String,

    pub pid: u32,
    pub infos: Vec<CommandInfo>,
}

impl CommandHist {
    pub fn new(category: &str, pid: u32) -> CommandHist {
        CommandHist {
            category: category.into(),
            pid: pid,
            infos: vec![],
        }
    }

    pub fn add(&mut self, command: &str) {
        self.infos.push(CommandInfo::new(command));
    }

    pub fn save(&mut self) {
        let mut text = String::new();
        for info in &self.infos {
            let time = info.time / 1000;
            let ymd = time / 1000_000;
            let hms = time % 1000_000;
            _ = writeln!(
                &mut text,
                "[{}_{:06}][{}]{}",
                ymd, hms, self.pid, info.command
            );
        }

        let file_path = Self::get_file_path(&self.category, true);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();
        file.write_all(text.as_ref()).unwrap()
    }

    fn get_file_path(category: &str, mkdirs: bool) -> String {
        let mut path = utils::get_appdata_local_dir();
        path.push_str("\\slickcmd");
        if mkdirs {
            win32::create_directory(&path);
        }
        path.push('\\');
        path.push_str(category);
        path.push_str(".history");
        path
    }

    fn parse_command_info(line: &str) -> Option<(u32, CommandInfo)> {
        let mut line = line.trim().to_owned();
        if line.is_empty() || line.as_bytes()[0] != b'[' {
            return None;
        }
        let index = line.find(']').unwrap_or_default();
        if index == 0 {
            return None;
        }
        let mut s_time = line[1..index].to_owned();
        s_time.remove(s_time.find('_').unwrap());

        let time = s_time.parse::<u64>().unwrap_or_default() * 1000;

        line = line[index + 1..].to_owned();
        if line.is_empty() || line.as_bytes()[0] != b'[' {
            return None;
        }
        let index = line.find(']').unwrap_or_default();
        if index == 0 {
            return None;
        }
        let s_pid = &line[1..index];
        let pid = s_pid.parse::<u32>().unwrap_or_default();
        let command = line[index + 1..].to_owned();

        let command_info = CommandInfo { time, command };
        Some((pid, command_info))
    }

    pub fn load_old_hists(category: &str) -> Vec<CommandHist> {
        let file_path = Self::get_file_path(category, false);
        if !utils::file_exists(&file_path) {
            return vec![];
        }
        let mut hists: Vec<CommandHist> = vec![];
        let mut hist: CommandHist = CommandHist::default();

        //
        let content = fs::read_to_string(Path::new(&file_path)).unwrap_or_default();
        let lines = content.split('\n');
        for line in lines {
            if let Some((pid, command_info)) = Self::parse_command_info(line) {
                if pid != hist.pid {
                    if hist.pid != 0 {
                        hists.push(hist);
                    }
                    hist = CommandHist::new(category, pid);
                }
                hist.infos.push(command_info);
            }
        }
        if hist.pid != 0 {
            hists.push(hist);
        }
        hists
    }

    pub fn is_empty(&self) -> bool {
        self.infos.is_empty()
    }
}
