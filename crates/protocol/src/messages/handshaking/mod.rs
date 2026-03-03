mod clientbound;
mod serverbound;

use protocol_derive::Readable;

pub use self::serverbound::*;

#[derive(Debug, Clone, Readable)]
pub enum ServerboundHandshakingMessage {
    #[discriminant(0x00)]
    Handshake(Handshake),
}
