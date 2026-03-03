use uuid::Uuid;

use crate::{Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct LoginStart {
    pub name: String,
    pub id: Uuid,
}
