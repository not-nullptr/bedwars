use crate::{GameProfile, Readable, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct LoginSuccess {
    pub profile: GameProfile,
}
