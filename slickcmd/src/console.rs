use crate::app_state::AppState;
use crate::command_hist::CommandHist;
use crate::command_hist_win::CommandHistWin;
use crate::dir_complete::DIR_COMPLETER;
use crate::dir_man::CurDir;
use crate::global::GLOBAL;
use crate::keyboard_input::KeyboardInput;
use crate::shell::{CmdShell, PsShell, Shell};
use slickcmd_common::consts::*;
use slickcmd_common::font_info::FontInfo;
use slickcmd_common::utils::iif;
use slickcmd_common::{logd, utils, win32};
use std::path::Path;
use std::{cell::RefCell, rc::Rc};
use widestring::U16CString;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::app::App;
use crate::win_man::WinMan;

#[derive(Default)]
pub struct Console {
    app_state: Rc<AppState>,

    pub hwnd: HWND,
    pub pid: u32,

    hwnd_term: HWND,

    hwnd_msg: HWND,
    shell: Box<dyn Shell>,

    cur_dir: CurDir,

    replacing_command: String,

    context: Rc<RefCell<ConsoleContext>>,

    command_hist: CommandHist,
    command_hist_win: Option<Box<CommandHistWin>>,

    last_command_y: i16,

    showing_ac_list: bool,
    update_cur_dir_on_key_up: bool,
    manual_cd_completing: bool,

}

#[derive(Default)]
struct ConsoleContext {
    h_stdout: HANDLE,
    h_stdin: HANDLE,
    attach_count: i32,
}

#[derive(Default)]
struct ConsoleDimensionInfo {
    window_col_count: i32,
    window_row_count: i32,
    cell_width: i32,
    cell_height: i32,
    cur_row: i32,
    cur_col: i32,
    cur_row_in_window: i32,
    cur_col_in_window: i32,
}

pub struct ConsoleAttach {
    context: Rc<RefCell<ConsoleContext>>,
    pub h_stdout: HANDLE,
    pub h_stdin: HANDLE,
}

impl ConsoleAttach {
    fn new(pid: u32, context: Rc<RefCell<ConsoleContext>>, with_stdin: bool) -> ConsoleAttach {
        let context0 = context.clone();
        let mut ctx = context.borrow_mut();
        let mut h_stdout: HANDLE = HANDLE::default();
        if ctx.attach_count == 0 {
            for _ in 0..50 {
                let attached = win32::attach_console(pid);
                if attached {
                    h_stdout = win32::get_std_handle(STD_OUTPUT_HANDLE);
                }
                if h_stdout.is_invalid() {
                    if attached {
                        win32::free_console();
                    }
                    win32::sleep(40);
                } else {
                    break;
                }
            }
            debug_assert!(!h_stdout.is_invalid());
            ctx.h_stdout = h_stdout;
        } else {
            h_stdout = ctx.h_stdout;
        }
        if with_stdin {
            ctx.h_stdin = win32::get_std_handle(STD_INPUT_HANDLE);
            let (_, ok) = win32::get_console_mode(ctx.h_stdin);
            if !ok {
                logd!("oops??");
                if ctx.h_stdin.is_invalid() {
                    logd!("invalid stdin?");
                }
            }
        }

        ctx.attach_count += 1;
        ConsoleAttach {
            context: context0,
            h_stdout,
            h_stdin: ctx.h_stdin,
        }
    }
}

impl Drop for ConsoleAttach {
    fn drop(&mut self) {
        let mut ctx = self.context.borrow_mut();
        ctx.attach_count -= 1;
        if ctx.attach_count == 0 {
            ctx.h_stdout = HANDLE::default();
            win32::set_std_handle(STD_OUTPUT_HANDLE, HANDLE::default());
            if !ctx.h_stdin.is_invalid() {
                win32::set_std_handle(STD_INPUT_HANDLE, HANDLE::default());
                ctx.h_stdin = HANDLE::default();
            }
            if !win32::free_console() {
                logd!("free console failed?");
            }
        }
    }
}

impl Console {
    pub fn new(hwnd: HWND, app_state: Rc<AppState>) -> Console {
        let (pid, _) = win32::get_window_thread_process_id(hwnd);

        let mut cur_dir = CurDir::default();

        let app_state2 = app_state.clone();
        cur_dir.on_set = Some(Box::new(move |dir| {
            app_state2.recent_dirs.use_dir(dir);
        }));

        //
        let hproc = win32::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        let exe_path = win32::get_module_file_name_ex(hproc);
        let exe_name = Path::new(&exe_path).file_name().unwrap_or_default();
        win32::close_handle(hproc);

        let shell: Box<dyn Shell> = if exe_name.eq_ignore_ascii_case("powershell.exe")
            || exe_name.eq_ignore_ascii_case("pwsh.exe")
        {
            Box::new(PsShell {})
        } else {
            Box::new(CmdShell {})
        };

        let command_hist = CommandHist::new(&shell.name(), 0);

        let console = Console {
            app_state,
            hwnd,
            hwnd_term: hwnd,
            cur_dir,
            pid,
            hwnd_msg: GLOBAL.hwnd_msg(),
            shell,
            command_hist,
            ..Default::default()
        };

        console
    }

    pub fn dispose(&mut self) {
        logd!("@ console dispose");
        if let Some(hist_win) = &mut self.command_hist_win {
            hist_win.destroy();
            self.command_hist_win = None;
        }
    }

    pub fn check_valid(&self) -> bool {
        win32::is_window(self.hwnd)
    }

    pub fn on_activate(&mut self) {
        win32::register_hotkey(self.hwnd_msg, 1, MOD_CONTROL | MOD_NOREPEAT, u32::from('L'));

        let hwnd_parent = win32::get_parent(self.hwnd);
        if !hwnd_parent.is_invalid() {
            self.hwnd_term = hwnd_parent;
        }; //for wt

        let _ca = self.new_console_attach(true);
        // self.new_console_attach(false); //?

        if !self.cur_dir.has_set() {
            self.update_cur_dir();
        }

    }

    pub fn on_deactivate(&mut self) {
        logd!("@ CONSOLE DEACTIVATE");
        if self.showing_ac_list {
            self.hide_ac_list();
        }
        win32::unregister_hotkey(self.hwnd_msg, 1);
        self.command_hist.save();
    }

    pub fn get_fg_pid(&self) -> u32 {
        let _ca = self.new_console_attach(false);
        let mut pids = [0u32; 4];
        let count = win32::get_console_process_list(&mut pids);
        let cur_pid = win32::get_current_process_id();
        let mut pid = self.pid;
        for n in 0..count as usize {
            if pids[n] != pid && pids[n] != cur_pid {
                pid = pids[n];
                break;
            }
        }
        pid
    }

    pub fn handle_key_down(&mut self, vk: VIRTUAL_KEY, alt_down: bool) -> bool {
        if !self.at_prompt() {
            return false;
        }

        if vk == VK_RETURN {
            return self.handle_return_down(alt_down);
        }

        false
    }

    fn handle_return_down(&mut self, _alt_down: bool) -> bool {
        let _ca = self.new_console_attach(false);

        let cur_y = self.get_y();
        let command_from_y = iif(cur_y > self.last_command_y, self.last_command_y + 1, 0);
        self.last_command_y = cur_y;

        let (_prompt, input) = self.read_prompt_input(command_from_y, false);
        logd!("PROMPT: {}, INPUT: {}", _prompt, input);

        if input.len() > 3 && input[..3].eq_ignore_ascii_case("cd ") && !self.is_cross_drive_cd() {
            let target_dir = input[3..].trim_start();
            if target_dir.len() >= 2 && target_dir.as_bytes()[1] == b':' {
                let cur_dir = self.resolve_cur_dir();
                if cur_dir.len() >= 2 && !cur_dir[..2].eq_ignore_ascii_case(&target_dir[..2]) {
                    self.replacing_command = String::from("cd /d ") + target_dir;
                }
            }
        }

        if !self.replacing_command.is_empty() {
            self.command_hist.add(&self.replacing_command);
            return true;
        } else if !input.is_empty() {
            self.command_hist.add(&input);
        }

        false
    }

    fn cd_complete(&mut self, input: &str) {
        if input.len() < 3 {
            return;
        }
        let mut input_dir = input[3..].trim_start();
        if input_dir.starts_with("/") {
            if input_dir.starts_with("/d ") {
                input_dir = input_dir[3..].trim_start()
            } else {
                return; //?
            }
        }
        if input_dir.starts_with('"') {
            input_dir = &input_dir[1..];
        }
        let dir = self.resolve_cur_dir();
        DIR_COMPLETER.complete(dir, input_dir.to_string());
    }

    pub fn handle_key_up(&mut self, vk: VIRTUAL_KEY, alt_down: bool) -> bool {
        if vk == VK_RETURN {
            if self.manual_cd_completing {
                self.manual_cd_completing = false;
            }
            return self.handle_return_up(alt_down);
        }

        if !self.at_prompt() {
            return false;
        }

        if self.update_cur_dir_on_key_up {
            self.update_cur_dir_on_key_up = false;
            self.update_cur_dir();
        }

        if !self.cur_dir.has_set() {
            self.update_cur_dir();
        }

        if alt_down {
            if self.handle_alt_key_up(vk) {
                return true;
            }
            if self.showing_ac_list && !self.manual_cd_completing {
                self.hide_ac_list();
            }
            return false;
        }

        //
        let (_, input) = self.read_prompt_input(0, true);
        let cding = input.starts_with("cd ");
        if self.manual_cd_completing && !cding {
            self.manual_cd_completing = false;
        }
        if cding && vk != VK_ESCAPE && (GLOBAL.options.cd_completion() || self.manual_cd_completing)
        {
            self.cd_complete(&input);
            return false;
        } else {
            if self.showing_ac_list {
                self.hide_ac_list();
            }
        }

        //
        false
    }

    pub fn hide_ac_list(&mut self) {
        if !self.showing_ac_list {
            logd!("(ac_list not showing)");
            return;
        }
        self.showing_ac_list = false;
        self.send_core_message(WM_SETTEXT, WPARAM(3), LPARAM(0));
    }

    fn get_font_info(&self, dim_info: &ConsoleDimensionInfo) -> FontInfo {
        let ca = self.new_console_attach(false);
        let mut cfi = CONSOLE_FONT_INFOEX::default();
        cfi.cbSize = size_of::<CONSOLE_FONT_INFOEX>() as _;
        if !win32::get_current_console_font_ex(ca.h_stdout, false, &mut cfi) {
            logd!("get console font failed.");
        }
        let mut fi = FontInfo::default();
        fi.width = App::dpi_aware_value(cfi.dwFontSize.X as _);
        fi.height = App::dpi_aware_value(cfi.dwFontSize.Y as _);

        fi.pitch_and_family = cfi.FontFamily as _;

        if cfi.FontWeight == 0 { //wt?
            fi.name = "Consolas".into();
            fi.width = dim_info.cell_width;
            fi.height = dim_info.cell_height;
        }
        else {
            let wsz_face_name = U16CString::from_vec_truncate(cfi.FaceName);
            fi.name = wsz_face_name.to_string_lossy();
        }
        fi
    }

    pub fn show_ac_list(&mut self, items: &[String]) -> LRESULT {
        if self.showing_ac_list {
            logd!("(already showing ac_list)");
        }
        self.showing_ac_list = true;

        let items_joined = items.join("\n");

        let mut data = String::new();

        let bounds = self.get_console_bounds();

        let size = (bounds.right - bounds.left, bounds.bottom - bounds.top);
        let dim_info = self.read_dimension_info(size);

        let (prompt, input) = self.read_prompt_input(0, true);
        if input.len() < 3 {
            logd!("invalid input while showing ac_list");
            return LRESULT(0);
        }
        let mut col = prompt.len();
        if input[3..].trim_start().starts_with("/d ") {
            col += "cd /d ".len();
        } else {
            col += "cd ".len();
        }

        let mut pt = POINT {
            x: col as i32 * dim_info.cell_width,
            y: dim_info.cur_row_in_window * dim_info.cell_height,
        };
        pt.x += bounds.left;
        pt.y += bounds.top;
        win32::client_to_screen(self.hwnd_term, &mut pt);

        data.push_str(&format!("{}\n{}\n{}\n", pt.x, pt.y, dim_info.cell_height));

        let fi = self.get_font_info(&dim_info);
        data.push_str(&fi.name);
        data.push('\n');

        data.push_str(&format!(
            "{}\n{}\n{}\n",
            fi.height, fi.width, fi.pitch_and_family
        ));

        data.push_str(&input);
        data.push('\n');

        data.push_str(&items_joined);

        let wsz_data = win32::wsz_from_str(&data);
        self.send_core_message(WM_SETTEXT, WPARAM(2), LPARAM(wsz_data.as_ptr() as _))
    }

    fn handle_alt_key_up(&mut self, vk: VIRTUAL_KEY) -> bool {
        match vk {
            VK_UP => self.on_alt_up(),
            VK_LEFT => self.on_alt_left(),
            VK_RIGHT => self.on_alt_right(),
            VK_DOWN => self.on_alt_down(),
            VK_HOME => self.on_alt_home(),
            VK_END => self.on_alt_end(),
            VK_F7 => self.on_alt_f7(),
            VK_F10 => self.on_alt_f10(),
            _ => {
                return false;
            }
        }
        true
    }

    fn ring_bell(&self) {
        win32::beep(1500, 300);
    }

    fn on_alt_up(&mut self) {
        self.update_cur_dir();
        let dir = self.cur_dir.go_up();

        if dir.is_empty() {
            self.ring_bell();
            return;
        }

        let cmd = "cd ..";

        let mut ki = KeyboardInput::new();
        ki.escape();
        ki.text(cmd);
        ki.enter();
        ki.send(true);

        //
        self.command_hist.add(cmd);
    }

    fn on_alt_left(&mut self) {
        let dir = self.cur_dir.go_back();
        if dir.is_empty() {
            self.ring_bell();
        } else {
            self.cd(&dir);
        }
    }

    fn on_alt_right(&mut self) {
        let dir = self.cur_dir.go_forward();
        if dir.is_empty() {
            self.ring_bell();
        } else {
            self.cd(&dir);
        }
    }

    fn on_alt_down(&mut self) {
        let (_, input) = self.read_prompt_input(0, false);
        if !input.starts_with("cd ") {
            self.set_input("cd ");
        }
        win32::post_message(
            self.hwnd_msg,
            WM_POST_ACTION,
            WPARAM(POST_ACTION_ALT_DOWN),
            LPARAM(self.hwnd_term.0 as _),
        );
    }

    fn on_post_alt_down(&mut self) {
        let mut _input: String = String::default();
        for _ in 1..50 {
            let (_, input) = self.read_prompt_input(0, true);
            if input.starts_with("cd ") {
                _input = input;
                break;
            }
            win32::sleep(40);
        }
        if _input.is_empty() {
            logd!("oops: expected cd input not found");
            return;
        }
        self.manual_cd_completing = true;
        self.cd_complete(&_input);
    }

    pub fn handle_post_action(&mut self, action: usize) {
        if action == POST_ACTION_ALT_DOWN {
            self.on_post_alt_down();
        }
    }

    fn get_console_bounds(&self) -> RECT {
        WinMan::get_console_bounds(self.hwnd_term)
    }

    fn on_alt_end(&mut self) {

        let bounds = self.get_console_bounds();

        let mut pt = {
            let _ca = self.new_console_attach(false);
            let size = (bounds.right - bounds.left, bounds.bottom - bounds.top);
            let dim_info = self.read_dimension_info(size);
            POINT {
                x: (dim_info.cur_col_in_window + 1) * dim_info.cell_width + 4,
                y: dim_info.cur_row_in_window * dim_info.cell_height,
            }
        };

        let mut pt_console = POINT{x: bounds.left, y: bounds.top};
        win32::client_to_screen(self.hwnd_term, &mut pt_console);
        pt.x += pt_console.x;
        pt.y += pt_console.y;

        let hmenu = win32::create_popup_menu();

        let recent_dirs = &self.app_state.recent_dirs;
        let count = recent_dirs.count();
        let max_count = GLOBAL.options.max_recent_dirs();
        let mut no = 1u32;
        for n in (0..count).rev() {
            let c_no = char::from_digit(no, max_count + 1).unwrap_or(' ');
            let item = format!("&{} {}\n", c_no, recent_dirs.at(n));
            win32::append_menu(hmenu, MF_STRING, 1000 + n as u16, Some(&item));
            no += 1;
            if no > max_count {
                break;
            }
        }
        if count == 0 {
            win32::append_menu(
                hmenu,
                MF_STRING | MF_DISABLED,
                1000,
                Some("(No Recent Dirs)"),
            );
        }

        logd!("PT.X: {}, PT.Y: {}", pt.x, pt.y);
        let lparam = LPARAM((pt.x << 16 | pt.y) as isize);
        self.post_core_message(WM_SHOW_MENU, WPARAM(hmenu.0 as usize), lparam);
    }

    pub fn handle_show_menu_result(&mut self, cmd: i32) {
        let item_index = cmd - 1000;
        if item_index >= 0 {
            let dir = self.app_state.recent_dirs.at(item_index as _);
            self.cur_dir.set(&dir);
            self.cd(&dir);
        }
    }

    fn post_core_message(&self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        let hwnd = win32::find_window_ex(
            HWND_MESSAGE,
            None,
            Some("slck_cmd_core_msg"),
            None,
        );
        win32::post_message(hwnd, msg, wparam, lparam);
    }

    fn send_core_message(&self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let hwnd = win32::find_window_ex(
            HWND_MESSAGE,
            None,
            Some("slck_cmd_core_msg"),
            None,
        );
        win32::send_message(hwnd, msg, wparam, lparam)
    }

    pub fn get_addr(&self) -> usize {
        self as *const _ as usize
    }

    fn on_alt_home(&mut self) {
        let dir = utils::get_home_dir();
        if dir == self.resolve_cur_dir() {
            logd!("(already at home)");
        } else {
            self.cur_dir.set(&dir);
            self.cd(&dir);
        }
    }

    fn on_alt_f7(&mut self) {
        if let Some(win) = &self.command_hist_win {
            win32::set_foreground_window(win.hwnd);
            return;
        }

        let category = &self.shell.name();
        let mut hists = CommandHist::load_old_hists(category, self.command_hist.sid());

        if !self.command_hist.is_empty() {
            hists.push(self.command_hist.clone());
        }

        let hists = hists.into_iter().map(|x| Rc::new(x)).collect::<Vec<_>>();

        let font_info = FontInfo {
            name: "Consolas".into(),
            height: 16,
            width: 8,
            pitch_and_family: 54,
        };
        let mut win = Box::new(CommandHistWin::new(self.hwnd));
        win.hists = hists;
        win.font_info = font_info;
        win.create(self.hwnd_term);
        self.command_hist_win = Some(win);
    }

    fn on_alt_f10(&mut self) {}

    fn handle_return_up(&mut self, _alt_down: bool) -> bool {
        if !self.replacing_command.is_empty() {
            let command = self.replacing_command.clone();
            self.replacing_command.clear();
            self.set_input(&command);

            let mut ki = KeyboardInput::new();
            ki.enter();
            ki.post(self.hwnd_term, false);

            self.update_cur_dir_on_key_up = true;

            return true;
        }
        self.update_cur_dir();
        false
    }

    fn cd(&mut self, dir: &str) {
        let cur_dir = self.resolve_cur_dir();
        if cur_dir == dir {
            logd!("(already at {})", cur_dir);
            return;
        }

        let mut cmd = String::from("cd ");
        if cur_dir.as_bytes()[0] != dir.as_bytes()[0] && !self.is_cross_drive_cd() {
            cmd.push_str("/d ");
        }
        let has_space = dir.contains(' ');
        if has_space {
            cmd.push('"');
        }
        cmd.push_str(dir);
        if has_space {
            cmd.push('"');
        }

        let mut ki = KeyboardInput::new();
        ki.escape();
        ki.text(&cmd);
        ki.enter();
        ki.send(true);

        //
        self.command_hist.add(&cmd);
    }

    pub fn clear(&self) {
        let mut ki = KeyboardInput::new();
        ki.escape();
        ki.text("cls");
        ki.enter();
        ki.send(true);
    }

    pub fn update_cur_dir(&mut self) {
        let dir = self.resolve_cur_dir();
        if dir.is_empty() {
            return; //?
        }
        self.cur_dir.set(&dir);
    }

    fn resolve_cur_dir(&self) -> String {
        let dir = self.shell.resolve_cur_dir(self);
        utils::normalize_dir_path(&dir)
    }

    fn at_prompt(&self) -> bool {
        self.shell.at_prompt(self)
    }

    pub fn is_line_editing(&self) -> bool {
        let ca = self.new_console_attach(true);
        let (mode, ok) = win32::get_console_mode(ca.h_stdin);
        if !ok {
            return false;
        }
        mode.contains(ENABLE_LINE_INPUT)
    }

    pub fn is_running_subprocess(&self) -> bool {
        let _ca = self.new_console_attach(false);
        let mut pids = [0u32; 4];
        let count = win32::get_console_process_list(&mut pids);
        if count == 0 {
            return false; //?
        }
        count > 2
    }

    pub fn new_console_attach(&self, with_stdin: bool) -> ConsoleAttach {
        ConsoleAttach::new(self.pid, self.context.clone(), with_stdin)
    }

    pub fn read_cur_line(&self, before_cursor: bool) -> String {
        let ca = self.new_console_attach(false);
        let mut csbi = CONSOLE_SCREEN_BUFFER_INFO::default();
        if !win32::get_console_screen_buffer_info(ca.h_stdout, &mut csbi) {
            logd!("get console screen buffer info failed.");
            return String::new();
        }
        let buf_size = if before_cursor {
            csbi.dwCursorPosition.X
        } else {
            csbi.dwSize.X
        };
        let mut line_buf = vec![0u16; buf_size as usize];
        let coord = COORD {
            X: 0,
            Y: csbi.dwCursorPosition.Y,
        };
        let cch = win32::read_console_output_character(ca.h_stdout, &mut line_buf, coord);
        let line = String::from_utf16_lossy(&line_buf[..cch as usize]);
        line
    }

    pub fn read_line(&self, y: i16) -> String {
        let ca = self.new_console_attach(false);
        let mut csbi = CONSOLE_SCREEN_BUFFER_INFO::default();
        if !win32::get_console_screen_buffer_info(ca.h_stdout, &mut csbi) {
            logd!("get console screen buffer info failed.");
            return String::new();
        }
        let buf_size = csbi.dwSize.X;
        let mut line_buf = vec![0u16; buf_size as usize];
        let y = if y < 0 {
            csbi.dwCursorPosition.Y + y
        } else {
            y
        };
        if y < 0 || y >= csbi.dwSize.Y {
            return String::new();
        }
        let coord = COORD { X: 0, Y: y };
        let cch = win32::read_console_output_character(ca.h_stdout, &mut line_buf, coord);
        let line = String::from_utf16_lossy(&line_buf[..cch as usize]);
        line
    }

    pub fn read_prompt_input(&self, from_y: i16, only_before_cursor: bool) -> (String, String) {
        let mut prompt = String::new();

        let ca = self.new_console_attach(true);
        let mut csbi = CONSOLE_SCREEN_BUFFER_INFO::default();
        if !win32::get_console_screen_buffer_info(ca.h_stdout, &mut csbi) {
            logd!("get console screen buffer info failed.");
            return (prompt, String::new());
        }

        let max_line_len = csbi.dwSize.X;
        let mut input: String = String::new();
        let mut line: String;
        let mut prompt_found: bool = false;

        let cur_x = csbi.dwCursorPosition.X;
        let cur_y = csbi.dwCursorPosition.Y;
        let before_y = if only_before_cursor {
            cur_y + 1
        } else {
            csbi.dwSize.Y
        };

        let line_before_cursor = {
            let mut buf = vec![0u16; cur_x as usize];
            let coord = COORD { X: 0, Y: cur_y };
            let cch = win32::read_console_output_character(ca.h_stdout, &mut buf, coord);
            String::from_utf16_lossy(&buf[..cch as usize])
        };

        for y in cur_y..before_y {
            if y == cur_y && only_before_cursor {
                line = line_before_cursor.clone();
            } else {
                let mut line_buf = vec![0u16; max_line_len as usize];
                let coord = COORD { X: 0, Y: y };
                let cch = win32::read_console_output_character(ca.h_stdout, &mut line_buf, coord);
                line = String::from_utf16_lossy(&line_buf[..cch as usize]);
                line = line.trim_end().into();
                if y == cur_y && line.len() < line_before_cursor.len() {
                    line = line_before_cursor.clone();
                }
            }
            if line.is_empty() {
                break;
            }
            if !prompt_found {
                prompt = self.parse_prompt(&line);
                if !prompt.is_empty() {
                    prompt_found = true;
                    line = line.chars().skip(prompt.chars().count()).collect();
                }
            }
            input.push_str(&line);
        }
        if prompt_found {
            return (prompt, input);
        }
        for y in (from_y..cur_y).rev() {
            let mut line_buf = vec![0u16; max_line_len as usize];
            let coord = COORD { X: 0, Y: y as i16 };
            let cch = win32::read_console_output_character(ca.h_stdout, &mut line_buf, coord);
            line = String::from_utf16_lossy(&line_buf[..cch as usize]);
            line = line.trim_end().into(); //xx?

            prompt = self.parse_prompt(&line);
            if !prompt.is_empty() {
                prompt_found = true;
                line = line.chars().skip(prompt.chars().count()).collect();
            }
            input.insert_str(0, &line);

            if prompt_found {
                input = self.clean_multiline_input_continuations(&input);
                return (prompt, input);
            }
        }
        (prompt, String::new())
    }

    fn clean_multiline_input_continuations(&self, s: &str) -> String {
        self.shell.clean_multiline_input_continuations(s)
    }

    fn parse_prompt(&self, line: &str) -> String {
        self.shell.parse_prompt(line)
    }

    fn read_dimension_info(&self, console_size: (i32, i32)) -> ConsoleDimensionInfo {
        let mut info = ConsoleDimensionInfo::default();
        let ca = self.new_console_attach(false);

        let mut csbi = CONSOLE_SCREEN_BUFFER_INFO::default();
        if !win32::get_console_screen_buffer_info(ca.h_stdout, &mut csbi) {
            logd!("get console screen buffer info failed.");
            return info;
        }

        info.cur_row = csbi.dwCursorPosition.Y as _;
        info.cur_col = csbi.dwCursorPosition.X as _;
        info.cur_row_in_window = info.cur_row - csbi.srWindow.Top as i32;
        info.cur_col_in_window = info.cur_col - csbi.srWindow.Left as i32;

        info.window_col_count = (csbi.srWindow.Right - csbi.srWindow.Left) as _;
        info.window_row_count = (csbi.srWindow.Bottom - csbi.srWindow.Top) as _;

        let client_width = console_size.0;
        let client_height = console_size.1;

        info.cell_width = (client_width as f64 / info.window_col_count as f64).floor() as _;
        info.cell_height = (client_height as f64 / info.window_row_count as f64).floor() as _;

        info
    }

    fn is_cross_drive_cd(&self) -> bool {
        self.shell.is_cross_drive_cd()
    }

    pub fn use_command(&mut self, command: &str, exec_now: bool) {
        //scope
        {
            let mut ki = KeyboardInput::new();
            ki.escape();
            ki.text(command);
            ki.send(true);
        }
        if exec_now {
            let mut ki = KeyboardInput::new();
            ki.enter();
            ki.post(self.hwnd_term, false);
        }
    }

    fn get_y(&self) -> i16 {
        let ca = self.new_console_attach(false);
        let mut csbi = CONSOLE_SCREEN_BUFFER_INFO::default();
        win32::get_console_screen_buffer_info(ca.h_stdout, &mut csbi);
        csbi.dwCursorPosition.Y
    }

    pub fn set_input(&mut self, input_text: &str) {
        self.shell.set_input(self, input_text);
    }

    pub fn notify_hist_win_destroyed(&mut self) {
        self.command_hist_win = None;
    }
}
