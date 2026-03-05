use generated::{Block, bedrock};

use crate::{
    HasData, HasDataKind, PaletteFormat, PaletteFormatKind, PalettedContainer, VarInt, Writable,
};

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

impl ChunkSection {
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        const AIR_BLOCKS: [u32; 3] = [
            generated::Block::Air(generated::air::Air {}).id(),
            generated::Block::CaveAir(generated::cave_air::CaveAir {}).id(),
            generated::Block::VoidAir(generated::void_air::VoidAir {}).id(),
        ];

        let is_air = matches!(block, Block::Air(_) | Block::CaveAir(_) | Block::VoidAir(_));
        let existing_air = AIR_BLOCKS.contains(
            &(self
                .block_states
                .palette_value_extend(y * 16 * 16 + z * 16 + x) as u32),
        );

        if is_air && !existing_air {
            self.block_count -= 1;
        } else if !is_air && existing_air {
            self.block_count += 1;
        }

        let index = y * 16 * 16 + z * 16 + x;
        self.block_states.set(index, block.id() as u16);
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        let index = y * 16 * 16 + z * 16 + x;
        let id = self.block_states.palette_value(index);
        Block::from_id(id as u32).unwrap_or(Block::Air(generated::air::Air {}))
    }
}

impl Chunk {
    pub fn generate() -> Self {
        let mut sections = Vec::new();
        let bedrock = Block::Bedrock(bedrock::Bedrock);
        for _ in 0..24 {
            let section = ChunkSection {
                block_count: 0,
                block_states: PalettedContainer::new(
                    PaletteFormatKind::Blocks,
                    PaletteFormat::SingleValue(VarInt::new(bedrock.id() as i32)),
                ),
                biomes: PalettedContainer::new(
                    PaletteFormatKind::Biomes,
                    PaletteFormat::SingleValue(VarInt::new(0)),
                ),
            };

            sections.push(section);
        }

        let chunk = Self { sections };

        // for x in 0..16 {
        //     for y in 0..16 {
        //         for z in 0..16 {
        //             chunk.set(x, y, z, Block::Bedrock(bedrock::Bedrock));
        //         }
        //     }
        // }

        chunk
    }

    pub fn empty() -> Self {
        let mut sections = Vec::new();
        for _ in 0..24 {
            let section = ChunkSection {
                block_count: 0,
                block_states: PalettedContainer::new(
                    PaletteFormatKind::Blocks,
                    PaletteFormat::HasData(HasData::new(
                        HasDataKind::Direct,
                        16 * 16 * 16,
                        PaletteFormatKind::Blocks,
                    )),
                ),
                biomes: PalettedContainer::new(
                    PaletteFormatKind::Biomes,
                    PaletteFormat::SingleValue(VarInt::new(0)),
                ),
            };

            sections.push(section);
        }

        Self { sections }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: Block) {
        let section_index = y / 16;
        let section_y = y % 16;
        tracing::info!(
            "setting block at ({}, {}, {}) in section {} to {:?}",
            x,
            y,
            z,
            section_index,
            block
        );
        self.sections[section_index].set(x, section_y, z, block);
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Block {
        let section_index = y / 16;
        let section_y = y % 16;
        self.sections[section_index].get(x, section_y, z)
    }
}
