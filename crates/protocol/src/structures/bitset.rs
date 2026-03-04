use protocol_derive::{Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct BitSet {
    pub bits: Vec<u64>,
}

impl BitSet {
    pub fn empty() -> Self {
        Self { bits: Vec::new() }
    }

    pub fn with_size(size: usize) -> Self {
        Self {
            bits: vec![0; (size + 63) / 64],
        }
    }

    pub fn toggle(&mut self, idx: usize) {
        let long_index = idx / 64;
        let bit_index = idx % 64;
        if long_index >= self.bits.len() {
            self.bits.resize(long_index + 1, 0);
        }
        self.bits[long_index] ^= 1 << bit_index;
    }
}
