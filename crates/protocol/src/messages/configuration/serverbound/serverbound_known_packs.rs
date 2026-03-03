use crate::{Readable, Writable, messages::configuration::KnownPack};

#[derive(Debug, Clone, Readable, Writable)]
pub struct ServerboundKnownPacks {
    pub known_packs: Vec<KnownPack>,
}
