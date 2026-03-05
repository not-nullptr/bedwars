use crate::{VarInt, Writable};

#[derive(Debug, Clone)]
pub struct PalettedContainer {
    pub kind: PaletteFormatKind,
    pub palette: PaletteFormat,
}

impl PalettedContainer {
    pub fn new(kind: PaletteFormatKind, palette: PaletteFormat) -> Self {
        Self { kind, palette }
    }

    pub fn set(&mut self, idx: usize, value: u16) {
        let bits_per_entry = self.palette.bits_per_entry(self.kind);
        match &mut self.palette {
            PaletteFormat::SingleValue(_) => panic!("Cannot set data on a single value palette"),
            PaletteFormat::HasData(d) => {
                // let entries_per_long = 64 / bits_per_entry as usize;
                // let long_index = idx / entries_per_long;
                // let bit_index = (idx % entries_per_long) * bits_per_entry as usize;
                // let entry_mask = ((1u64 << bits_per_entry) - 1) << bit_index;
                // if long_index >= d.data.len() {
                //     panic!("index out of bounds for palette data array");
                // }
                // d.data[long_index] &= !entry_mask;
                // d.data[long_index] |= (value as u64) << bit_index;

                match &mut d.kind {
                    HasDataKind::Direct => {
                        let entries_per_long = 64 / bits_per_entry as usize;
                        let long_index = idx / entries_per_long;
                        let bit_index = (idx % entries_per_long) * bits_per_entry as usize;
                        let entry_mask = ((1u64 << bits_per_entry) - 1) << bit_index;
                        if long_index >= d.data.len() {
                            panic!("index out of bounds for palette data array");
                        }
                        d.data[long_index] &= !entry_mask;
                        d.data[long_index] |= (value as u64) << bit_index;
                    }

                    HasDataKind::Indirect(_) => {
                        todo!("setting data on an indirect palette is not yet implemented")
                    }
                }
            }
        }
    }

    pub fn palette_value(&self, idx: usize) -> u16 {
        let bits_per_entry = self.palette.bits_per_entry(self.kind);
        match &self.palette {
            PaletteFormat::SingleValue(value) => value.value() as u16,
            PaletteFormat::HasData(d) => match &d.kind {
                HasDataKind::Direct => {
                    let entries_per_long = 64 / bits_per_entry as usize;
                    let long_index = idx / entries_per_long;
                    let bit_index = (idx % entries_per_long) * bits_per_entry as usize;
                    let entry_mask = ((1u64 << bits_per_entry) - 1) << bit_index;
                    if long_index >= d.data.len() {
                        panic!("index out of bounds for palette data array");
                    }
                    ((d.data[long_index] & entry_mask) >> bit_index) as u16
                }

                HasDataKind::Indirect(indirect) => {
                    let entries_per_long = 64 / bits_per_entry as usize;
                    let long_index = idx / entries_per_long;
                    let bit_index = (idx % entries_per_long) * bits_per_entry as usize;
                    let entry_mask = ((1u64 << bits_per_entry) - 1) << bit_index;
                    if long_index >= d.data.len() {
                        panic!("index out of bounds for palette data array");
                    }
                    let palette_index = ((d.data[long_index] & entry_mask) >> bit_index) as usize;
                    if palette_index >= indirect.palette.len() {
                        panic!("index out of bounds for indirect palette");
                    }
                    indirect.palette[palette_index].value() as u16
                }
            },
        }
    }

    pub fn palette_value_extend(&mut self, idx: usize) -> u16 {
        let bits_per_entry = self.palette.bits_per_entry(self.kind);
        match &mut self.palette {
            PaletteFormat::SingleValue(value) => value.value() as u16,
            PaletteFormat::HasData(d) => match &d.kind {
                HasDataKind::Direct => {
                    let entries_per_long = 64 / bits_per_entry as usize;
                    let long_index = idx / entries_per_long;
                    let bit_index = (idx % entries_per_long) * bits_per_entry as usize;
                    let entry_mask = ((1u64 << bits_per_entry) - 1) << bit_index;
                    if long_index >= d.data.len() {
                        d.data.resize(long_index + 1, 0);
                    }
                    ((d.data[long_index] & entry_mask) >> bit_index) as u16
                }

                HasDataKind::Indirect(indirect) => {
                    let entries_per_long = 64 / bits_per_entry as usize;
                    let long_index = idx / entries_per_long;
                    let bit_index = (idx % entries_per_long) * bits_per_entry as usize;
                    let entry_mask = ((1u64 << bits_per_entry) - 1) << bit_index;
                    if long_index >= d.data.len() {
                        panic!("index out of bounds for palette data array");
                    }
                    let palette_index = ((d.data[long_index] & entry_mask) >> bit_index) as usize;
                    if palette_index >= indirect.palette.len() {
                        panic!("index out of bounds for indirect palette");
                    }
                    indirect.palette[palette_index].value() as u16
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaletteFormat {
    HasData(HasData),
    SingleValue(VarInt),
}

#[derive(Debug, Clone)]
pub struct HasData {
    pub kind: HasDataKind,
    pub data: Vec<u64>,
}

impl HasData {
    pub fn new(kind: HasDataKind, num_entries: usize, format: PaletteFormatKind) -> Self {
        Self {
            data: vec![0; kind.necessary_elements(num_entries, format)],
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HasDataKind {
    Indirect(Indirect),
    Direct,
}

impl HasDataKind {
    pub fn bits_per_entry(&self, kind: PaletteFormatKind) -> u8 {
        match kind {
            PaletteFormatKind::Blocks => match self {
                HasDataKind::Indirect(indirect) => indirect.bits_per_entry,
                HasDataKind::Direct => 15,
            },
            PaletteFormatKind::Biomes => match self {
                HasDataKind::Indirect(indirect) => indirect.bits_per_entry,
                HasDataKind::Direct => 7,
            },
        }
    }

    pub fn necessary_elements(&self, desired: usize, kind: PaletteFormatKind) -> usize {
        let bits_per_entry = self.bits_per_entry(kind);
        let entries_per_long = 64 / bits_per_entry as usize;
        (desired + entries_per_long - 1) / entries_per_long
    }
}

impl PaletteFormat {
    pub fn bits_per_entry(&self, kind: PaletteFormatKind) -> u8 {
        match kind {
            PaletteFormatKind::Blocks => match self {
                PaletteFormat::SingleValue(_) => 0,
                PaletteFormat::HasData(d) => d.kind.bits_per_entry(kind),
            },
            PaletteFormatKind::Biomes => match self {
                PaletteFormat::SingleValue(_) => 0,
                PaletteFormat::HasData(d) => d.kind.bits_per_entry(kind),
            },
        }
    }
    pub fn necessary_elements(&self, desired: usize) -> usize {
        let bits_per_entry = self.bits_per_entry(PaletteFormatKind::Blocks);
        let entries_per_long = 64 / bits_per_entry as usize;
        (desired + entries_per_long - 1) / entries_per_long
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PaletteFormatKind {
    Blocks,
    Biomes,
}

#[derive(Debug, Clone)]
pub struct Indirect {
    pub bits_per_entry: u8,
    pub palette: Vec<VarInt>,
}

impl Indirect {
    pub fn new(bits_per_entry: u8, palette: Vec<VarInt>) -> Self {
        Self {
            bits_per_entry,
            palette,
        }
    }
}

impl Writable for PalettedContainer {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let bits_per_entry = self.palette.bits_per_entry(self.kind);
        bits_per_entry.write_into(writer).await?;
        match &self.palette {
            PaletteFormat::SingleValue(value) => value.write_into(writer).await?,
            PaletteFormat::HasData(d) => {
                match &d.kind {
                    HasDataKind::Indirect(indirect) => {
                        indirect.palette.write_into(writer).await?;
                    }
                    HasDataKind::Direct => {}
                }

                // VERSION SPECIFIC: 1.21.5+ no longer sends the length of the data array
                for entry in &d.data {
                    entry.write_into(writer).await?;
                }
            }
        }

        Ok(())
    }
}
