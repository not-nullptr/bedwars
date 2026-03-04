#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarInt(i32);

impl VarInt {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> i32 {
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

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}
