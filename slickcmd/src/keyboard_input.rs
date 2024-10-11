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

    has_key: bool,

    expect_ctrl_down: bool,
    expect_alt_down: bool,
    expect_shift_down: bool,
}

pub struct PostKeyboardInput {
    pub hwnd_target: HWND,
    pub keyboard_input: KeyboardInput
}

pub struct PostKeyboardInputs(RefCell<VecDeque<PostKeyboardInput>>);

pub static POST_KEYBOARD_INPUTS: PostKeyboardInputs = PostKeyboardInputs::new();

unsafe impl Send for PostKeyboardInputs{}
unsafe impl Sync for PostKeyboardInputs{}

impl PostKeyboardInputs {
    const fn new() -> PostKeyboardInputs {
        PostKeyboardInputs(RefCell::new(VecDeque::new()))
    }

    pub fn add(&self, hwnd_target: HWND, keyboard_input: KeyboardInput) {
        self.0.borrow_mut().push_back(PostKeyboardInput{
            hwnd_target,
            keyboard_input
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

    pub fn expect_modifiers_state(&mut self, ctrl_down: bool, alt_down: bool, shift_down: bool) {
        self.expect_ctrl_down = ctrl_down;
        self.expect_alt_down = alt_down;
        self.expect_shift_down = shift_down;
    }

    pub fn key_down(&mut self, vk: VIRTUAL_KEY) {
        Self::_key_down(vk, &mut self.inputs);
        self.has_key = true;
    }

    fn _key_down(vk: VIRTUAL_KEY, inputs: &mut Vec<INPUT>) {
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wVk = vk;
        inputs.push(input);
    }

    pub fn key_up(&mut self, vk: VIRTUAL_KEY) {
        Self::_key_up(vk, &mut self.inputs);
        self.has_key = true;
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

    pub fn char_down(&mut self, c: char) {
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wScan = c as u32 as u16;
        input.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE;
        self.inputs.push(input);
    }

    pub fn char_up(&mut self, c: char) {
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wScan = c as u32 as u16;
        input.Anonymous.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;
        self.inputs.push(input);
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
        let mut mod_inputs = Vec::<INPUT>::new();
        let mut mod_input_count = 0;

        if self.has_key {
            let cur_ctrl_down = win32::get_async_key_state(VK_LCONTROL) < 0;
            let cur_alt_down = win32::get_async_key_state(VK_LMENU) < 0;
            let cur_shift_down = win32::get_async_key_state(VK_LSHIFT) < 0;

            if self.expect_ctrl_down != cur_ctrl_down {
                if cur_ctrl_down {
                    Self::_key_up(VK_LCONTROL, &mut mod_inputs);
                } else {
                    Self::_key_down(VK_LCONTROL, &mut mod_inputs);
                }
            }
            if self.expect_alt_down != cur_alt_down {
                if cur_alt_down {
                    Self::_key_up(VK_LMENU, &mut mod_inputs);
                } else {
                    Self::_key_down(VK_LMENU, &mut mod_inputs);
                }
            }
            if self.expect_shift_down != cur_shift_down {
                if cur_shift_down {
                    Self::_key_up(VK_LSHIFT, &mut mod_inputs);
                } else {
                    Self::_key_down(VK_LSHIFT, &mut mod_inputs);
                }
            }
            mod_input_count = mod_inputs.len();
        }
        if mod_input_count != 0 {
            inputs.append(&mut mod_inputs.clone());
        }
        inputs.append(&mut self.inputs.clone());
        if mod_input_count != 0 {
            mod_inputs.reverse();
            for input in mod_inputs.iter_mut() {
                unsafe {
                    input.Anonymous.ki.dwFlags.0 ^= KEYEVENTF_KEYUP.0;
                }
            }
            inputs.append(&mut mod_inputs);
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
        self.has_key = false;
    }
}
