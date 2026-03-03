use serde::Serialize;

use crate::{Identifier, Readable, Writable};

#[derive(Debug, Clone, Serialize, Readable, Writable)]
pub struct RegistryData {
    pub registry_id: Identifier,
    pub entries: Vec<RegistryEntry>,
}

impl RegistryData {
    pub fn new(registry_id: Identifier, entries: Vec<RegistryEntry>) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

#[derive(Debug, Clone, Serialize, Readable, Writable)]
pub struct RegistryEntry {
    pub entry_id: Identifier,
    pub nbt: Option<fastnbt::Value>,
}

impl RegistryEntry {
    pub fn new(entry_id: Identifier, nbt: Option<fastnbt::Value>) -> Self {
        Self { entry_id, nbt }
    }
}
