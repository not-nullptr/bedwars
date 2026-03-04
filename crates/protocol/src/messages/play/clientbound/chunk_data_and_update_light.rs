use crate::{BitSet, ChunkData, VarInt, Writable};

#[derive(Debug, Clone, Writable)]
pub struct ChunkDataAndUpdateLight {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub data: ChunkData,
    pub light: LightData,
}

#[derive(Debug, Clone, Writable)]
pub struct LightData {
    pub sky_light_mask: BitSet,
    pub block_light_mask: BitSet,
    pub empty_sky_light_mask: BitSet,
    pub empty_block_light_mask: BitSet,
    pub sky_light_arrays: Vec<LightArray>,
    pub block_light_arrays: Vec<LightArray>,
}

#[derive(Debug, Clone, Writable)]
pub struct LightArray {
    pub data: Vec<u8>,
}
