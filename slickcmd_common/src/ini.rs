use crate::win32;
use std::any::TypeId;
use std::env;
use std::fmt::Display;
use std::str::FromStr;

pub struct Ini {
    file_path: String,
}

impl Ini {
    pub fn new(file_name: Option<&str>) -> Ini {
        let mut file_path: String;
        if let Some(file_name) = file_name {
            file_path = file_name.into();
        } else {
            file_path = env::current_exe().unwrap().to_string_lossy().into();
            let pos = file_path.rfind('.').unwrap();
            file_path = file_path[..pos].to_string();
        }
        if !file_path.ends_with(".ini") {
            file_path.push_str(".ini");
        }
        Ini { file_path }
    }

    pub fn write<T: Display>(&self, section: &str, key: &str, value: T) {
        let value = value.to_string();
        win32::write_private_profile_string(section, key, &value, &self.file_path);
    }

    fn _read(&self, section: &str, key: &str, def_val: Option<&str>) -> String {
        win32::get_private_profile_string(section, key, def_val, &self.file_path)
    }

    pub fn read<T: Default + FromStr>(&self, section: &str, key: &str) -> T {
        let value = self._read(section, key, None);
        value.parse().unwrap_or_default()
    }

    pub fn read_or<T>(&self, section: &str, key: &str, default: T) -> T
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        if TypeId::of::<T>() == TypeId::of::<String>() {
            let value = self._read(section, key, Some("\n"));
            return if value == "\n" {
                default
            } else {
                T::from_str(&value).unwrap()
            };
        }
        let value = self._read(section, key, None);
        value.parse().unwrap_or(default)
    }
}
