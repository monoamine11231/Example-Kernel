use crate::input::key_codes::{
    get_from_code_pressed, get_from_code_released, KeyPressedCodes, KeyReleasedCodes,
};

use super::key_codes;

type Callback = fn(key_code: i32);

// NOT THREAD SAFE - needs to be fixed if more threads are added
pub static mut KEYBOARD: Keyboard = Keyboard {
    shift: false,
    caps: false,
    alt: false,
    ctrl: false,
    callback0: NOOP,
    callback1: NOOP,
    callback2: NOOP,
    callback3: NOOP,
    callback4: NOOP,
};

pub const NOOP: fn(i32) = |_| {};

pub struct Keyboard {
    callback0: Callback,
    callback1: Callback,
    callback2: Callback,
    callback3: Callback,
    callback4: Callback,
    shift: bool,
    caps: bool,
    alt: bool,
    ctrl: bool,
}

impl Keyboard {
    pub fn set_callback0(&mut self, callback: Callback) {
        self.callback0 = callback;
    }
    pub fn set_callback1(&mut self, callback: Callback) {
        self.callback1 = callback;
    }
    pub fn set_callback2(&mut self, callback: Callback) {
        self.callback2 = callback;
    }
    pub fn set_callback3(&mut self, callback: Callback) {
        self.callback3 = callback;
    }
    pub fn set_callback4(&mut self, callback: Callback) {
        self.callback4 = callback;
    }
    pub fn handle_key(&mut self, code: i32) {
        (self.callback0)(code);
        (self.callback1)(code);
        (self.callback2)(code);
        (self.callback3)(code);
        (self.callback4)(code);

        match get_from_code_pressed(code) {
            KeyPressedCodes::CapsLock => self.caps = !self.caps,
            KeyPressedCodes::LeftShift => self.shift = true,
            KeyPressedCodes::RightShift => self.shift = true,
            KeyPressedCodes::LeftCtrl => self.ctrl = true,
            KeyPressedCodes::RightCtrl => self.ctrl = true,
            KeyPressedCodes::LeftAlt => self.alt = true,
            KeyPressedCodes::RightAlt => self.alt = true,
            _ => (),
        }

        match get_from_code_released(code) {
            KeyReleasedCodes::LeftShift => self.shift = false,
            KeyReleasedCodes::RightShift => self.shift = false,
            KeyReleasedCodes::LeftCtrl => self.ctrl = false,
            KeyReleasedCodes::RightCtrl => self.ctrl = false,
            KeyReleasedCodes::LeftAlt => self.alt = false,
            KeyReleasedCodes::RightAlt => self.alt = false,
            _ => (),
        }
    }
}

pub fn get_key_pressed(key_code: i32) -> KeyPressedCodes {
    return get_from_code_pressed(key_code);
}

pub fn get_key_released(key_code: i32) -> KeyReleasedCodes {
    return get_from_code_released(key_code);
}
