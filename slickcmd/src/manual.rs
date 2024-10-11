use slickcmd_common::{win32, winproc};
use std::thread;
use windows::Win32::System::Console::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const MANUAL: &'static str = r"
      _           _
     /_`/._  /_  / `_ _   _/
    ._////_ /\  /_,/ / //_/


    ALT + ↑      Go To Parent Dir
    ALT + ←      Go Back
    ALT + →      Go Forward
    ALT + ↓      Go To a Sub Dir
    ALT + HOME    Go To Home Dir
    ALT + END     Go To a Recent Dir

    ALT + F7      Show Command History
    CTRL+ L       Clear Screen



Press any key to close..";

pub fn show() {
    thread::spawn(_show);
    winproc::message_loop(HACCEL::default());
}

fn _show() {
    win32::alloc_console();
    win32::set_console_title("Slick Cmd Manual");

    let hstdout = win32::get_std_handle(STD_OUTPUT_HANDLE);
    win32::write_console(hstdout, MANUAL);

    let hstdin = win32::get_std_handle(STD_INPUT_HANDLE);
    let mut input_records = [INPUT_RECORD::default(); 1];
    let mut read_count = 0u32;

    loop {
        win32::read_console_input(hstdin, &mut input_records, &mut read_count);
        if input_records[0].EventType == KEY_EVENT as _ {
            let key_event = unsafe { input_records[0].Event.KeyEvent };
            let key_down = key_event.bKeyDown.as_bool();
            let vk = key_event.wVirtualKeyCode;
            if key_down && vk != VK_CONTROL.0 && vk != VK_MENU.0 {
                win32::exit_process(0);
                break;
            }
        }
    }
}
