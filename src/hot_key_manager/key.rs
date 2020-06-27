use strum_macros::EnumString;

#[derive(Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, EnumString, Display, Debug)]
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
}
