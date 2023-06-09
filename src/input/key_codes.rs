// Scan code set 1

#[derive(Copy, Clone)]
pub enum KeyPressedCodes {
    A = 0x1E,
    B = 0x30,
    C = 0x2E,
    D = 0x20,
    E = 0x12,
    F = 0x21,
    G = 0x22,
    H = 0x23,
    I = 0x17,
    J = 0x24,
    K = 0x25,
    L = 0x26,
    M = 0x32,
    N = 0x31,
    O = 0x18,
    P = 0x19,
    Q = 0x10,
    R = 0x13,
    S = 0x1F,
    T = 0x14,
    U = 0x16,
    V = 0x2F,
    W = 0x11,
    X = 0x2D,
    Y = 0x15,
    Z = 0x2C,
    One = 0x02,
    Two = 0x03,
    Three = 0x04,
    Four = 0x05,
    Five = 0x06,
    Six = 0x07,
    Seven = 0x08,
    Eight = 0x09,
    Nine = 0x0A,
    Zero = 0x0B,
    Enter = 0x1C,
    Space = 0x39,
    Backspace = 0x0E,
    CapsLock = 0x3A,
    LeftShift = 0x2A,
    RightShift = 0x36,
    LeftCtrl = 0x1D,
    RightCtrl = 0x9D,
    LeftAlt = 0x38,
    RightAlt = 0xB8,
    Unknown = 0x00,
}

#[derive(Copy, Clone)]
pub enum KeyReleasedCodes {
    A = 0x9E,
    B = 0xB0,
    C = 0xAE,
    D = 0xA0,
    E = 0x92,
    F = 0xA1,
    G = 0xA2,
    H = 0xA3,
    I = 0x97,
    J = 0xA4,
    K = 0xA5,
    L = 0xA6,
    M = 0xB2,
    N = 0xB1,
    O = 0x98,
    P = 0x99,
    Q = 0x90,
    R = 0x93,
    S = 0x9F,
    T = 0x94,
    U = 0x96,
    V = 0xAF,
    W = 0x91,
    X = 0xAD,
    Y = 0x95,
    Z = 0xAC,
    One = 0x82,
    Two = 0x83,
    Three = 0x84,
    Four = 0x85,
    Five = 0x86,
    Six = 0x87,
    Seven = 0x88,
    Eight = 0x89,
    Nine = 0x8A,
    Zero = 0x8B,
    Enter = 0x9C,
    Space = 0xB9,
    Backspace = 0x8E,
    CapsLock = 0xBA,
    LeftShift = 0xAA,
    RightShift = 0xB6,
    LeftCtrl = 0x9D,
    RightCtrl = 0xDD,
    LeftAlt = 0xB8,
    RightAlt = 0xD8,
    Unknown = 0x00,
}

pub fn get_from_code_pressed(key_code: i32) -> KeyPressedCodes {
    match key_code {
        0x1E => KeyPressedCodes::A,
        0x30 => KeyPressedCodes::B,
        0x2E => KeyPressedCodes::C,
        0x20 => KeyPressedCodes::D,
        0x12 => KeyPressedCodes::E,
        0x21 => KeyPressedCodes::F,
        0x22 => KeyPressedCodes::G,
        0x23 => KeyPressedCodes::H,
        0x17 => KeyPressedCodes::I,
        0x24 => KeyPressedCodes::J,
        0x25 => KeyPressedCodes::K,
        0x26 => KeyPressedCodes::L,
        0x32 => KeyPressedCodes::M,
        0x31 => KeyPressedCodes::N,
        0x18 => KeyPressedCodes::O,
        0x19 => KeyPressedCodes::P,
        0x10 => KeyPressedCodes::Q,
        0x13 => KeyPressedCodes::R,
        0x1F => KeyPressedCodes::S,
        0x14 => KeyPressedCodes::T,
        0x16 => KeyPressedCodes::U,
        0x2F => KeyPressedCodes::V,
        0x11 => KeyPressedCodes::W,
        0x2D => KeyPressedCodes::X,
        0x15 => KeyPressedCodes::Y,
        0x2C => KeyPressedCodes::Z,
        0x02 => KeyPressedCodes::One,
        0x03 => KeyPressedCodes::Two,
        0x04 => KeyPressedCodes::Three,
        0x05 => KeyPressedCodes::Four,
        0x06 => KeyPressedCodes::Five,
        0x07 => KeyPressedCodes::Six,
        0x08 => KeyPressedCodes::Seven,
        0x09 => KeyPressedCodes::Eight,
        0x0A => KeyPressedCodes::Nine,
        0x0B => KeyPressedCodes::Zero,
        0x1C => KeyPressedCodes::Enter,
        0x39 => KeyPressedCodes::Space,
        0x0E => KeyPressedCodes::Backspace,
        0x3A => KeyPressedCodes::CapsLock,
        0x2A => KeyPressedCodes::LeftShift,
        0x36 => KeyPressedCodes::RightShift,
        0x1D => KeyPressedCodes::LeftCtrl,
        0x9D => KeyPressedCodes::RightCtrl,
        0x38 => KeyPressedCodes::LeftAlt,
        0xB8 => KeyPressedCodes::RightAlt,
        _ => KeyPressedCodes::Unknown,
    }
}

pub fn get_from_code_released(key_code: i32) -> KeyReleasedCodes {
    match key_code {
        0x9E => KeyReleasedCodes::A,
        0xB0 => KeyReleasedCodes::B,
        0xAE => KeyReleasedCodes::C,
        0xA0 => KeyReleasedCodes::D,
        0x92 => KeyReleasedCodes::E,
        0xA1 => KeyReleasedCodes::F,
        0xA2 => KeyReleasedCodes::G,
        0xA3 => KeyReleasedCodes::H,
        0x97 => KeyReleasedCodes::I,
        0xA4 => KeyReleasedCodes::J,
        0xA5 => KeyReleasedCodes::K,
        0xA6 => KeyReleasedCodes::L,
        0xB2 => KeyReleasedCodes::M,
        0xB1 => KeyReleasedCodes::N,
        0x98 => KeyReleasedCodes::O,
        0x99 => KeyReleasedCodes::P,
        0x90 => KeyReleasedCodes::Q,
        0x93 => KeyReleasedCodes::R,
        0x9F => KeyReleasedCodes::S,
        0x94 => KeyReleasedCodes::T,
        0x96 => KeyReleasedCodes::U,
        0xAF => KeyReleasedCodes::V,
        0x91 => KeyReleasedCodes::W,
        0xAD => KeyReleasedCodes::X,
        0x95 => KeyReleasedCodes::Y,
        0xAC => KeyReleasedCodes::Z,
        0x82 => KeyReleasedCodes::One,
        0x83 => KeyReleasedCodes::Two,
        0x84 => KeyReleasedCodes::Three,
        0x85 => KeyReleasedCodes::Four,
        0x86 => KeyReleasedCodes::Five,
        0x87 => KeyReleasedCodes::Six,
        0x88 => KeyReleasedCodes::Seven,
        0x89 => KeyReleasedCodes::Eight,
        0x8A => KeyReleasedCodes::Nine,
        0x8B => KeyReleasedCodes::Zero,
        0x9C => KeyReleasedCodes::Enter,
        0xB9 => KeyReleasedCodes::Space,
        0x8E => KeyReleasedCodes::Backspace,
        0xBA => KeyReleasedCodes::CapsLock,
        0xAA => KeyReleasedCodes::LeftShift,
        0xB6 => KeyReleasedCodes::RightShift,
        0x9D => KeyReleasedCodes::LeftCtrl,
        0xDD => KeyReleasedCodes::RightCtrl,
        0xB8 => KeyReleasedCodes::LeftAlt,
        0xD8 => KeyReleasedCodes::RightAlt,
        _ => KeyReleasedCodes::Unknown,
    }
}
