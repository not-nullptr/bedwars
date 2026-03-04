use crate::{Readable, VarInt, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct SetCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
