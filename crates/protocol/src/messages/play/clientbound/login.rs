use crate::{Gamemode, Identifier, Position, Readable, VarInt, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct Login {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub dimension_names: Vec<String>,
    pub max_players: VarInt,
    pub view_distance: VarInt,
    pub simulation_distance: VarInt,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub do_limited_crafting: bool,
    pub dimension_type: VarInt,
    pub dimension_name: Identifier,
    pub hashed_seed: i64,
    pub game_mode: Gamemode,
    pub previous_game_mode: u8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub death_info: Option<DeathInfo>,
    pub portal_cooldown: VarInt,
    pub sea_level: VarInt,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, Readable, Writable)]
pub struct DeathInfo {
    pub death_dimension_name: Identifier,
    pub death_location: Position,
}
