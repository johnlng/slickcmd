use anyhow::Result;
use slickcmd_common::{utils, win32};
use std::path::Path;
use std::{env, fs};
use widestring::U16CStr;
use windows::core::{w, Interface, PCWSTR};
use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
use windows::Win32::System::Com::{IPersistFile, STGM_READ};
use windows::Win32::UI::Shell::{FOLDERID_Startup, IShellLinkW, ShellLink, KF_FLAG_DEFAULT};
use crate::global::GLOBAL;

pub struct StartupLink();

impl StartupLink {
    fn get_startup_dir() -> String {
        win32::sh_get_known_folder_path(&FOLDERID_Startup, KF_FLAG_DEFAULT)
    }

    pub fn sync_state(run_on_startup: bool) {
        let link = Self::get_existing_link();
        if link.is_ok() {
            let (link_path, exe_path) = link.unwrap();
            if !run_on_startup {
                _ = fs::remove_file(link_path);
            } else if !exe_path.eq_ignore_ascii_case(&utils::get_exe_path()) {
                _ = fs::remove_file(link_path);
                _ = Self::create();
            }
        } else if run_on_startup {
            _ = Self::create();
        }
    }

    pub fn exists() -> Result<bool> {
        Ok(!Self::get_existing_link()?.0.is_empty())
    }

    //(link_path, target_path)
    pub fn get_existing_link() -> Result<(String, String)> {
        let dir = Self::get_startup_dir();
        let dir = Path::new(&dir).read_dir()?;

        let shell_link = win32::co_create_instance::<IShellLinkW>(&ShellLink)?;
        let ppf: IPersistFile = shell_link.cast()?;

        let exe_name = env::current_exe()?.file_name().unwrap().to_string_lossy().into_owned();
        for entry in dir {
            let path: String;
            if let Ok(entry) = entry {
                path = entry.path().to_string_lossy().into_owned();
            } else {
                continue;
            }
            if !path.ends_with(".lnk") {
                continue;
            }
            let wsz_path = win32::wsz_from_str(&path);
            unsafe {
                if ppf.Load(win32::pwsz(&wsz_path), STGM_READ).is_err() {
                    continue;
                }
                if shell_link.Resolve(GLOBAL.hwnd_main(), 0).is_err() {
                    continue;
                }
                let mut wsz_exe_path = [0u16; 256];
                let mut fd = WIN32_FIND_DATAW::default();
                if shell_link.GetPath(&mut wsz_exe_path, &mut fd as _, 0).is_err() {
                    continue;
                }
                let exe_path = U16CStr::from_slice_truncate(&wsz_exe_path)?.to_string_lossy();
                let pos = exe_path.rfind('\\').unwrap_or_default();
                if exe_path[pos + 1..].eq_ignore_ascii_case(&exe_name) {
                    return Ok((path, exe_path));
                }
            }
        }
        Ok((String::new(), String::new()))
    }

    pub fn create() -> Result<()> {
        let shell_link = win32::co_create_instance::<IShellLinkW>(&ShellLink)?;
        let exe_path = env::current_exe()?.to_string_lossy().into_owned();
        let wsz_exe_path = win32::wsz_from_str(&exe_path);
        unsafe {
            shell_link.SetPath(PCWSTR(wsz_exe_path.as_ptr()))?;
            shell_link.SetDescription(w!("Slick Cmd"))?;
        }

        let dir = Self::get_startup_dir();
        let link_path = dir + "\\Slick Cmd.lnk";
        let wsz_link_path = win32::wsz_from_str(&link_path);

        let ppf: IPersistFile = shell_link.cast()?;
        unsafe {
            ppf.Save(PCWSTR(wsz_link_path.as_ptr()), true)?;
        }

        Ok(())
    }

    pub fn remove() -> bool {
        let (link_path, _) = Self::get_existing_link().unwrap_or_default();
        if utils::file_exists(&link_path) {
            fs::remove_file(&link_path).is_ok()
        } else {
            false
        }
    }
}
