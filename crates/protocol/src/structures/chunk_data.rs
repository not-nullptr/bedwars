use crate::{PalettedContainer, VarInt, Writable};

#[derive(Debug, Clone, Writable)]
pub struct ChunkData {
    pub heightmaps: Vec<()>, // TODO
    pub data: Chunk,
    pub block_entities: Vec<BlockEntity>,
}

#[derive(Debug, Clone, Writable)]
pub struct BlockEntity {
    pub packed_xz: u8,
    pub y: u16,
    pub kind: VarInt,
    pub data: fastnbt::Value,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub sections: Vec<ChunkSection>,
}

impl Writable for Chunk {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let mut bytes = Vec::new();
        for section in &self.sections {
            section.write_into(&mut bytes).await?;
        }

        bytes.write_into(writer).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Writable)]
pub struct ChunkSection {
    pub block_count: u16,
    pub block_states: PalettedContainer,
    pub biomes: PalettedContainer,
}
