#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarInt(u32);

impl VarInt {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }

    pub const fn len_bytes(self) -> usize {
        let mut num = self.0;
        let mut len = 0;
        loop {
            len += 1;
            num >>= 7;
            if num == 0 {
                break;
            }
        }
        len
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<VarInt> for u32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}
