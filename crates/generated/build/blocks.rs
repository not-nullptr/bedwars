use std::collections::HashMap;

use serde::Deserialize;

pub type BlockMap = HashMap<String, BlockEntry>;

#[derive(Debug, Clone, Deserialize)]
pub struct BlockEntry {
    pub properties: Option<HashMap<String, Vec<String>>>,
    pub states: Vec<BlockEntryState>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockEntryState {
    pub id: u32,
    pub properties: Option<HashMap<String, String>>,
}
