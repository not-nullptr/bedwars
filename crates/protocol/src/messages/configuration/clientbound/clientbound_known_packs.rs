use crate::{Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct ClientboundKnownPacks {
    pub known_packs: Vec<KnownPack>,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct KnownPack {
    pub namespace: String,
    pub pathname: String,
    pub version: String,
}
