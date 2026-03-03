use crate::{Readable, Writable};
use uuid::Uuid;

#[derive(Debug, Clone, Readable, Writable)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}
