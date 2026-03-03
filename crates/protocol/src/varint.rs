#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarInt(u32);

impl VarInt {
    pub const fn encode(value: u32) -> Self {
        let mut value = value;
        let mut bytes = [0u8; 5];
    }
}

impl Into<u32> for VarInt {
    fn into(self) -> u32 {
        self.into_inner()
    }
}

impl 