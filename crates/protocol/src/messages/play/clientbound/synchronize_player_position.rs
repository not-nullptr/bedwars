use crate::{Readable, TeleportFlags, VarInt, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct SynchronizePlayerPosition {
    pub teleport_id: VarInt,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub velocity_x: f64,
    pub velocity_y: f64,
    pub velocity_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: TeleportFlags,
}
