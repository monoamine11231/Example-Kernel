use crate::input::key_codes::get_from_code;

use crate::input::key_codes::KeyPressedCodes;

type Callback = fn(key_code: i32);

// NOT THREAD SAFE - needs to be fixed if more threads are added
pub static mut KEYBOARD: Keyboard = Keyboard {
    shift: false,
    caps: false,
    alt: false,
    ctrl: false,
    callback: NOOP,
};

pub const NOOP: fn(i32) = |_| {};

pub struct Keyboard {
    callback: Callback,
    shift: bool,
    caps: bool,
    alt: bool,
    ctrl: bool,
}

impl Keyboard {
    pub fn set_callback(&mut self, callback: Callback) {
        self.callback = callback;
    }

    pub fn handle_key(&mut self, code: i32) {
        (self.callback)(code);
    }
}

pub fn get_key(key_code: i32) -> KeyPressedCodes {
    return get_from_code(key_code);
}
