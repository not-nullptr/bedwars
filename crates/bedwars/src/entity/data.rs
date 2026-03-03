use crate::entity::player::PlayerEntity;

#[derive(Debug, Clone)]
pub enum EntityData {
    Player(PlayerEntity),
}
