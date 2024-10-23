use crate::GLOBAL;
use slickcmd_common::consts::WM_POST_ACTION;
use slickcmd_common::win32;
use std::cell::RefCell;
use std::collections::VecDeque;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Default)]
pub struct KeyboardInput {
    inputs: Vec<INPUT>,
}

pub struct PostKeyboardInput {
    pub hwnd_target: HWND,
    pub keyboard_input: KeyboardInput,
}

pub struct PostKeyboardInputs(RefCell<VecDeque<PostKeyboardInput>>);

pub static POST_KEYBOARD_INPUTS: PostKeyboardInputs = PostKeyboardInputs::new();

unsafe impl Send for PostKeyboardInputs {}
unsafe impl Sync for PostKeyboardInputs {}

impl PostKeyboardInputs {
    const fn new() -> PostKeyboardInputs {
        PostKeyboardInputs(RefCell::new(VecDeque::new()))
    }

    pub fn add(&self, hwnd_target: HWND, keyboard_input: KeyboardInput) {
        self.0.borrow_mut().push_back(PostKeyboardInput {
            hwnd_target,
            keyboard_input,
        });
    }

    pub fn fetch(&self) -> Option<PostKeyboardInput> {
        self.0.borrow_mut().pop_front()
    }
}

impl KeyboardInput {
    pub fn new() -> KeyboardInput {
        KeyboardInput::default()
    }

    pub fn key_down(&mut self, vk: VIRTUAL_KEY) {
        Self::_key_down(vk, &mut self.inputs);
    }

    fn _key_down(vk: VIRTUAL_KEY, inputs: &mut Vec<INPUT>) {
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wVk = vk;
        inputs.push(input);
    }

    pub fn key_up(&mut self, vk: VIRTUAL_KEY) {
        Self::_key_up(vk, &mut self.inputs);
    }

    fn _key_up(vk: VIRTUAL_KEY, inputs: &mut Vec<INPUT>) {
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wVk = vk;
        input.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
        inputs.push(input);
    }

    pub fn key_press(&mut self, vk: VIRTUAL_KEY) {
        self.key_down(vk);
        self.key_up(vk);
    }

    pub fn escape(&mut self) {
        self.key_press(VK_ESCAPE);
    }

    pub fn enter(&mut self) {
        self.key_press(VK_RETURN);
    }

    fn char_input_by_vk(&mut self, vk: u8, shift_state: u8, key_up: bool) {
        if !key_up {
            if shift_state & 1 != 0 {
                self.key_down(VK_LSHIFT);
            }
            if shift_state & 2 != 0 {
                self.key_down(VK_LCONTROL);
            }
            if shift_state & 4 != 0 {
                self.key_down(VK_LMENU);
            }
            if shift_state & 8 != 0 {
                self.key_down(VK_KANA);
            }
        }
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wVk = VIRTUAL_KEY(vk as _);
        if key_up {
            input.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
        }
        self.inputs.push(input);
        if key_up {
            if shift_state & 8 != 0 {
                self.key_up(VK_KANA);
            }
            if shift_state & 4 != 0 {
                self.key_up(VK_LMENU);
            }
            if shift_state & 2 != 0 {
                self.key_up(VK_LCONTROL);
            }
            if shift_state & 1 != 0 {
                self.key_up(VK_LSHIFT);
            }
        }
    }

    fn char_down(&mut self, c: char) {
        let mut wcs = [0u16; 2];
        let wcs = c.encode_utf16(&mut wcs);
        if wcs.len() == 1 {
            let vk = win32::vk_key_scan(wcs[0]);
            let shift_state = (vk >> 8) as i8;
            if shift_state != -1 {
                return self.char_input_by_vk(vk as _, shift_state as _, false);
            }
        }
        for wc in wcs {
            let mut input = INPUT::default();
            input.r#type = INPUT_KEYBOARD;
            input.Anonymous.ki.wScan = *wc;
            input.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE;
            self.inputs.push(input);
        }
    }

    fn char_up(&mut self, c: char) {
        let mut wcs = [0u16; 2];
        let wcs = c.encode_utf16(&mut wcs);
        if wcs.len() == 1 {
            let vk = win32::vk_key_scan(wcs[0]);
            let shift_state = (vk >> 8) as i8;
            if shift_state != -1 {
                return self.char_input_by_vk(vk as _, shift_state as _, true);
            }
        }
        for wc in wcs {
            let mut input = INPUT::default();
            input.r#type = INPUT_KEYBOARD;
            input.Anonymous.ki.wScan = *wc;
            input.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
            self.inputs.push(input);
        }
    }

    pub fn char_press(&mut self, c: char) {
        self.char_down(c);
        self.char_up(c);
    }

    pub fn text(&mut self, text: &str) {
        for c in text.chars() {
            self.char_press(c);
        }
    }

    pub fn send(&mut self) {
        let mut inputs = Vec::<INPUT>::new();

        let has_key = self.inputs.iter().any(|x| {
            !unsafe { x.Anonymous.ki }.dwFlags.contains(KEYEVENTF_UNICODE)
        });

        let cur_ctrl_down = win32::get_async_key_state(VK_LCONTROL) < 0;
        let cur_alt_down = win32::get_async_key_state(VK_LMENU) < 0;
        let cur_shift_down = win32::get_async_key_state(VK_LSHIFT) < 0;

        if has_key {
            if cur_ctrl_down {
                Self::_key_up(VK_LCONTROL, &mut inputs);
            }
            if cur_alt_down {
                Self::_key_up(VK_LMENU, &mut inputs);
            }
            if cur_shift_down {
                Self::_key_up(VK_LSHIFT, &mut inputs);
            }
        }
        inputs.append(&mut self.inputs.clone());

        if has_key {
            if cur_ctrl_down {
                Self::_key_down(VK_LCONTROL, &mut inputs);
            }
            if cur_alt_down {
                Self::_key_down(VK_LMENU, &mut inputs);
            }
            if cur_shift_down {
                Self::_key_down(VK_LSHIFT, &mut inputs);
            }
        }
        win32::send_input(&inputs);
    }

    pub fn post(self, hwnd_target: HWND) {
        POST_KEYBOARD_INPUTS.add(hwnd_target, self);
        let hwnd = GLOBAL.hwnd_msg();
        win32::post_message(hwnd, WM_POST_ACTION, WPARAM(0), LPARAM(0));
    }

    pub fn clear(&mut self) {
        self.inputs.clear();
    }
}
