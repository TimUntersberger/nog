use bitflags::bitflags;

bitflags!{
    #[derive(Default)]
    #[allow(dead_code)]
    pub struct Modifier: u32 {
        const ALT = 0x0001;
        const CONTROL = 0x0002;
        const SHIFT = 0x0004;
        const WIN = 0x0008;
    }
}