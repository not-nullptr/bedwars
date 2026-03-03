use crate::{Readable, Writable};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Readable, Writable)]
#[net_repr(u8)]
pub enum Gamemode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}
