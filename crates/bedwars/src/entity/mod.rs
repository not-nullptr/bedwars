pub mod allocator;
pub mod player;

mod data;

pub use data::EntityData;

use crate::entity::allocator::EntityAllocator;

pub struct Entity {
    pub id: i32,
    pub data: EntityData,
}

impl Entity {
    pub fn new(allocator: &mut EntityAllocator, data: EntityData) -> Self {
        Self {
            id: allocator.allocate(),
            data,
        }
    }
}
