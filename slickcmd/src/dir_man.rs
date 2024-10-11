use crate::GLOBAL;
use itertools::Itertools;
use slickcmd_common::utils::iif;
use slickcmd_common::{logd, utils, win32};
use std::cell::RefCell;
use std::path::Path;
use std::{cmp, fs};

#[derive(Default)]
pub struct CurDir {
    dirs: Vec<String>,
    dir_index: usize,

    pub on_set: Option<Box<dyn Fn(&str)>>,
}

impl CurDir {
    pub fn _inspect(&self) -> Vec<String> {
        let iter = self.dirs.iter().enumerate();
        iter.map(|(n, v)| v.clone() + iif(n == self.dir_index, "@", "")).collect()
    }

    pub fn _get_index(&self) -> usize {
        self.dir_index
    }

    pub fn get(&self) -> String {
        if self.dirs.is_empty() {
            return String::new();
        }
        self.dirs[self.dir_index].clone()
    }

    pub fn set(&mut self, dir: &str) {
        logd!("@ Set cur dir: {}", dir);

        debug_assert!(dir.ends_with("\\"));

        let dir: String = dir.into();

        if self.dirs.is_empty() {
            self.dirs.push(dir.clone());
            self.dir_index = 0;
        } else if self.dirs[self.dir_index] != dir {
            if self.dir_index < self.dirs.len() - 1 {
                self.dir_index += 1;
                self.dirs[self.dir_index] = dir.clone();
                self.dirs.truncate(self.dir_index + 1);
            } else {
                self.dirs.push(dir.clone());
                self.dir_index = self.dirs.len() - 1;
            }
        }
        //
        if let Some(on_set) = &self.on_set {
            on_set(&dir);
        }
    }

    pub fn has_set(&self) -> bool {
        !self.dirs.is_empty()
    }

    pub fn go_back(&mut self) -> String {
        if self.dir_index == 0 {
            return String::new();
        }
        self.dir_index -= 1;
        self.dirs[self.dir_index].clone()
    }

    pub fn go_forward(&mut self) -> String {
        if self.dir_index >= self.dirs.len() - 1 {
            return String::new();
        }
        self.dir_index += 1;
        self.dirs[self.dir_index].clone()
    }

    pub fn go_up(&mut self) -> String {
        let dir = self.get();
        let mut chars: Vec<_> = dir.chars().collect();
        let mut len = chars.len();
        if len == 0 {
            return String::new();
        }
        if chars[len - 1] == '\\' {
            chars.remove(len - 1);
            len -= 1;
        }

        for n in (0..len).rev() {
            if chars[n] == '\\' {
                chars.truncate(n + 1);
                let dir = String::from_iter(chars.iter());
                self.set(&dir);
                return dir;
            }
        }
        String::new()
    }
}

#[derive(Default)]
pub struct RecentDirs(RefCell<Vec<String>>);

//Dirs are arranged in reverse order of recency
impl RecentDirs {
    fn get_file_path(mkdirs: bool) -> String {
        let mut path = utils::get_appdata_local_dir();
        path.push_str("\\slickcmd");
        if mkdirs {
            win32::create_directory(&path);
        }
        path.push('\\');
        path.push_str("dirs.txt");
        path
    }

    pub fn load(&self) {
        let file_path = Self::get_file_path(false);
        if !utils::file_exists(&file_path) {
            return;
        }
        let content = fs::read_to_string(Path::new(&file_path)).unwrap_or_default();
        *self.0.borrow_mut() = content
            .split('\n')
            .into_iter()
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect();
    }

    pub fn save(&self) {
        let file_path = Self::get_file_path(true);
        let deque = self.0.borrow();
        let max_len = GLOBAL.options.max_recent_dirs();
        let skip_count = cmp::max(deque.len() as i32 - max_len as i32, 0);
        let content = deque.iter().skip(skip_count as _).join("\n");
        _ = fs::write(file_path, content);
    }

    pub fn use_dir(&self, dir: &str) {
        let mut dirs = self.0.borrow_mut();
        let mut index: usize = dirs.len();

        for (n, t_dir) in dirs.iter().enumerate() {
            if t_dir.eq_ignore_ascii_case(dir) {
                index = n;
                break;
            }
        }
        if index < dirs.len() {
            dirs.remove(index);
        }
        dirs.push(dir.into());
    }

    pub fn count(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn at(&self, index: usize) -> String {
        if let Some(dir) = self.0.borrow().get(index) {
            return dir.clone();
        }
        String::new()
    }

    pub fn _inspect(&self) -> Vec<String> {
        self.0.borrow().clone()
    }
}
