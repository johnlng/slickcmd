use slickcmd_common::consts::*;
use slickcmd_common::{logd, win32};
use std::cmp::Ordering::Equal;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use widestring::U16CString;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use crate::global::GLOBAL;

#[derive(Default)]
pub struct DirCompleter();

pub static DIR_COMPLETER: DirCompleter = DirCompleter {};

static CUR_REQ_ID: AtomicU32 = AtomicU32::new(0);

struct CompleteRequest {
    id: u32,
    cur_dir: String,
    input_dir: String,
}

impl DirCompleter {
    fn complete_thread_proc(req: CompleteRequest) {
        let mut input_dir = req.input_dir;
        let mut cur_dir = req.cur_dir;
        let file_name: String;
        if input_dir.len() < 2 || input_dir.as_bytes()[1] != b':' {
            if !cur_dir.ends_with('\\') {
                cur_dir.push('\\');
            }
            file_name = cur_dir + &input_dir;
        } else {
            if input_dir.len() == 2 && input_dir.as_bytes()[1] == b':' {
                input_dir.push('\\');
            }
            file_name = input_dir.clone();
        }
        let mut fd = WIN32_FIND_DATAW::default();
        let hfind = win32::find_first_file_ex(
            &(file_name + "*"),
            FindExInfoBasic,
            &mut fd,
            FindExSearchLimitToDirectories,
            FIND_FIRST_EX_LARGE_FETCH,
        );
        if hfind.is_invalid() {
            if req.id == CUR_REQ_ID.load(Relaxed) {
                win32::send_message(
                    GLOBAL.hwnd_msg(),
                    WM_HIDE_AUTO_COMPLETE,
                    WPARAM(0),
                    LPARAM(0),
                );
            }
            return;
        }
        let rpos = input_dir.rfind('\\');
        let parent_end_pos = if rpos.is_none() { 0 } else { rpos.unwrap() + 1 };
        let parent_input_dir = input_dir[..parent_end_pos].to_owned();

        let mut items = Vec::<(String, String)>::new();
        const DOT: u16 = '.' as _;

        while CUR_REQ_ID.load(Relaxed) == req.id {
            let mut skip = false;
            let attrs = FILE_FLAGS_AND_ATTRIBUTES(fd.dwFileAttributes);
            if !attrs.contains(FILE_ATTRIBUTE_DIRECTORY) {
                skip = true;
            } else if fd.cFileName[0] == DOT
                && (fd.cFileName[1] == 0 || fd.cFileName[1] == DOT && fd.cFileName[2] == 0)
            {
                skip = true;
            }
            if !skip {
                let wsz_file_name = U16CString::from_vec_truncate(&fd.cFileName);
                let item = parent_input_dir.clone() + &wsz_file_name.to_string_lossy();
                let item_lowercase = item.to_ascii_lowercase();
                items.push((item, item_lowercase));
            }
            if !win32::find_next_file(hfind, &mut fd) {
                break;
            }
        }
        win32::find_close(hfind);

        items.sort_by(|a, b| {
            let cmp = a.1.cmp(&b.1);
            if cmp == Equal {
                a.0.cmp(&b.0)
            } else {
                cmp
            }
        });
        let items: Vec<String> = items.into_iter().map(|x| x.0).collect();
        let count = items.len();
        if req.id != CUR_REQ_ID.load(Relaxed) {
            return;
        }
        if count == 0 {
            win32::send_message(
                GLOBAL.hwnd_msg(),
                WM_HIDE_AUTO_COMPLETE,
                WPARAM(0),
                LPARAM(0),
            );
        } else {
            win32::send_message(
                GLOBAL.hwnd_msg(),
                WM_SHOW_AUTO_COMPLETE,
                WPARAM(items.len()),
                LPARAM(items.as_ptr() as isize),
            );
        }
    }

    pub fn complete(&self, cur_dir: String, input_dir: String) {
        logd!("INPUT_DIR: {}", input_dir);
        let id = CUR_REQ_ID.fetch_add(1, Relaxed) + 1;
        let req = CompleteRequest {
            id,
            cur_dir,
            input_dir,
        };
        thread::spawn(move || Self::complete_thread_proc(req));
    }
}
