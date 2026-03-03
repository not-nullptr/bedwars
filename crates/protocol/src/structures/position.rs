#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl Position {
    pub fn new(x: i32, y: i16, z: i32) -> Self {
        Self { x, y, z }
    }
}
