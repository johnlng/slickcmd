use crate::console::Console;
use crate::key_hook_suppressor::KeyHookSuppressor;
use slickcmd_common::{utils, win32};
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub trait Shell {
    fn name(&self) -> String;
    fn at_prompt(&self, console: &Console) -> bool;

    fn resolve_cur_dir(&self, console: &Console) -> String;

    fn parse_prompt(&self, line: &str) -> String;

    fn is_cross_drive_cd(&self) -> bool;

    fn clean_multiline_input_continuations(&self, input: &str) -> String;

    fn set_input(&self, console: &Console, input_text: &str);

}

impl Default for Box<dyn Shell> {
    fn default() -> Box<dyn Shell> {
        Box::new(CmdShell {})
    }
}

pub struct CmdShell();

impl Shell for CmdShell {
    fn name(&self) -> String {
        "cmd".into()
    }

    fn at_prompt(&self, console: &Console) -> bool {
        let _ca = console.new_console_attach(true);
        console.is_line_editing() && !console.is_running_subprocess()
    }

    fn resolve_cur_dir(&self, console: &Console) -> String {
        utils::get_working_dir(console.pid)
    }

    fn parse_prompt(&self, line: &str) -> String {
        let pos = line.find('>').unwrap_or_default();
        if pos == 0 {
            return String::new();
        }
        let prompt = line[..pos + 1].to_string();
        let dir = &prompt[..pos];
        if !utils::dir_exists(dir) {
            return String::new();
        }
        prompt
    }

    fn is_cross_drive_cd(&self) -> bool {
        false
    }

    fn clean_multiline_input_continuations(&self, input: &str) -> String {
        input.replace("^More? ", "")
    }

    fn set_input(&self, console: &Console, input_text: &str) {

        let _khs = KeyHookSuppressor::new(console.hwnd);
        let ca = console.new_console_attach(true);

        let wc_input_text: Vec<u16> = input_text.encode_utf16().collect();
        let cch = wc_input_text.len();
        let mut inputs = vec![INPUT_RECORD::default(); cch * 2 + 2];

        inputs[0].EventType = KEY_EVENT as _;
        let key_event = unsafe {&mut inputs[0].Event.KeyEvent};
        key_event.bKeyDown = TRUE;
        key_event.wVirtualKeyCode = 27;
        key_event.wRepeatCount = 1;

        inputs[1].EventType = KEY_EVENT as _;
        let key_event = unsafe {&mut inputs[1].Event.KeyEvent};
        key_event.bKeyDown = FALSE;
        key_event.wVirtualKeyCode = 27;
        key_event.wRepeatCount = 1;

        for n in 0..cch {
            let c = wc_input_text[n];
            let ir_down = &mut inputs[(n + 1) * 2];

            ir_down.EventType = KEY_EVENT as _;
            let key_event = unsafe {&mut ir_down.Event.KeyEvent};
            key_event.bKeyDown = TRUE;
            key_event.uChar.UnicodeChar = c;
            key_event.wRepeatCount = 1;

            let ir_up = &mut inputs[(n + 1) * 2 + 1];
            ir_up.EventType = KEY_EVENT as _;
            let key_event = unsafe {&mut ir_up.Event.KeyEvent};
            key_event.bKeyDown = FALSE;
            key_event.uChar.UnicodeChar = c;
            key_event.wRepeatCount = 1;
        }

        let mut written_count = 0u32;
        win32::write_console_input(ca.h_stdin, &inputs, &mut written_count);
    }
}

pub struct PsShell();

impl Shell for PsShell {
    fn name(&self) -> String {
        "ps".into()
    }

    fn at_prompt(&self, console: &Console) -> bool {
        !console.is_running_subprocess()
    }

    fn resolve_cur_dir(&self, console: &Console) -> String {
        for y in (-10..1).rev() {
            let line = if y == 0 {
                console.read_cur_line(true)
            } else {
                console.read_line(y)
            };
            if line.starts_with("PS ") {
                let pos = line.find('>').unwrap_or_default();
                if pos != 0 {
                    return line[3..pos].to_string();
                }
            }
        }
        String::new() //?
    }

    fn parse_prompt(&self, line: &str) -> String {
        let pos = line.find('>').unwrap_or_default();
        if pos == 0 {
            return String::new();
        }
        if line.len() < pos + 2 { //?
            return String::new();
        }
        let prompt = line[..pos + 2].to_string();
        if !prompt.starts_with("PS ") {
            return String::new();
        }
        let dir = &prompt[3..pos];
        if !utils::dir_exists(dir) {
            return String::new();
        }
        prompt
    }

    fn is_cross_drive_cd(&self) -> bool {
        true
    }

    fn clean_multiline_input_continuations(&self, input: &str) -> String {
        input.replace("`>> ", "")
    }

    fn set_input(&self, console: &Console, input_text: &str) {
        let _khs = KeyHookSuppressor::new(console.hwnd);
        win32::send_message(console.hwnd, WM_CHAR, WPARAM('\x1b' as _), LPARAM(0));
        let wcs = input_text.encode_utf16();
        for wc in wcs {
            win32::send_message(console.hwnd, WM_CHAR, WPARAM(wc as _), LPARAM(0));
        }
    }
}
