use strum_macros::{AsRefStr, Display, EnumString};
use winapi::um::winuser::*;

#[derive(
    Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, EnumString, AsRefStr, Display, Debug,
)]
#[allow(dead_code)]
pub enum Key {
    Enter = 0x0D,
    Plus = 0xBB,
    Minus = 0xBD,
    A = 0x41,
    B = 0x42,
    C = 0x43,
    D = 0x44,
    E = 0x45,
    F = 0x46,
    G = 0x47,
    H = 0x48,
    I = 0x49,
    J = 0x4A,
    K = 0x4B,
    L = 0x4C,
    M = 0x4D,
    N = 0x4E,
    O = 0x4F,
    P = 0x50,
    Q = 0x51,
    R = 0x52,
    S = 0x53,
    T = 0x54,
    U = 0x55,
    V = 0x56,
    W = 0x57,
    X = 0x58,
    Y = 0x59,
    Z = 0x5A,
    Tab = VK_TAB as isize,
    Space = VK_SPACE as isize,
    F1 = VK_F1 as isize,
    F2 = VK_F2 as isize,
    F3 = VK_F3 as isize,
    F4 = VK_F4 as isize,
    F5 = VK_F5 as isize,
    F6 = VK_F6 as isize,
    F7 = VK_F7 as isize,
    F8 = VK_F8 as isize,
    F9 = VK_F9 as isize,
    F10 = VK_F10 as isize,
    F11 = VK_F11 as isize,
    F12 = VK_F12 as isize,
    #[strum(serialize = ",")]
    Comma = VK_OEM_COMMA as isize,
    #[strum(serialize = ".")]
    Period = VK_OEM_PERIOD as isize,
    #[strum(serialize = "Shift")]
    LShift = VK_LSHIFT as isize,
    #[strum(serialize = "Control")]
    LControl = VK_LCONTROL as isize,
    #[strum(serialize = "Alt")]
    LAlt = VK_LMENU as isize,
    Escape = VK_ESCAPE as isize,
    Backspace = VK_BACK as isize,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    #[strum(serialize = "0")]
    Zero = 0x30,
    #[strum(serialize = "1")]
    One = 0x31,
    #[strum(serialize = "2")]
    Two = 0x32,
    #[strum(serialize = "3")]
    Three = 0x33,
    #[strum(serialize = "4")]
    Four = 0x34,
    #[strum(serialize = "5")]
    Five = 0x35,
    #[strum(serialize = "6")]
    Six = 0x36,
    #[strum(serialize = "7")]
    Seven = 0x37,
    #[strum(serialize = "8")]
    Eight = 0x38,
    #[strum(serialize = "9")]
    Nine = 0x39,
    OEM_1 = VK_OEM_1 as isize,
    OEM_2 = VK_OEM_2 as isize,
    OEM_3 = VK_OEM_3 as isize,
    OEM_4 = VK_OEM_4 as isize,
    OEM_5 = VK_OEM_5 as isize,
    OEM_6 = VK_OEM_6 as isize,
    OEM_7 = VK_OEM_7 as isize,
    OEM_8 = VK_OEM_8 as isize,
    OEM_102 = VK_OEM_102 as isize,
}
