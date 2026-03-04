use bitflags::bitflags;
use protocol_derive::{Readable, Writable};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Readable, Writable)]
    pub struct TeleportFlags: i32 {
        const RELATIVE_X = 0x0001;
        const RELATIVE_Y = 0x0002;
        const RELATIVE_Z = 0x0004;
        const RELATIVE_YAW = 0x0008;
        const RELATIVE_PITCH = 0x0010;
        const RELATIVE_VELOCITY_X = 0x0020;
        const RELATIVE_VELOCITY_Y = 0x0040;
        const RELATIVE_VELOCITY_Z = 0x0080;
        const ROTATE_VELOCITY = 0x0100;
    }
}
