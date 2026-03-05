use generated::Block;
use protocol::Chunk;
use std::collections::HashMap;

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new() -> Self {
        let mut chunks = HashMap::new();
        for x in -1..=1 {
            for z in -1..=1 {
                let mut chunk = Chunk::empty();
                // set bottom layer to bedrock
                let bedrock = Block::Bedrock(generated::bedrock::Bedrock);
                for x in 0..16 {
                    for z in 0..16 {
                        chunk.set(x, 0, z, bedrock);
                    }
                }
                chunks.insert((x, z), chunk);
            }
        }
        Self { chunks }
    }
}
