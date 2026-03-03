use crate::{Identifier, Readable, VarInt, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct UpdateTags {
    pub tagged_registries: Vec<TaggedRegistry>,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct TaggedRegistry {
    pub registry: Identifier,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct Tag {
    pub tag_name: Identifier,
    pub entries: Vec<VarInt>,
}
