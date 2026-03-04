mod chunk_data_and_update_light;
mod game_event;
mod login;
mod set_center_chunk;
mod synchronize_player_position;

pub use chunk_data_and_update_light::*;
pub use game_event::*;
pub use login::*;
pub use set_center_chunk::*;
pub use synchronize_player_position::*;

use crate::Writable;

#[derive(Debug, Clone, Writable)]
pub enum ClientboundPlayMessage {
    #[discriminant(0x26)]
    GameEvent(GameEvent),

    #[discriminant(0x2C)]
    ChunkDataAndUpdateLight(ChunkDataAndUpdateLight),

    #[discriminant(0x30)]
    Login(Login),

    #[discriminant(0x46)]
    SynchronizePlayerPosition(SynchronizePlayerPosition),

    #[discriminant(0x5C)]
    SetCenterChunk(SetCenterChunk),
}
