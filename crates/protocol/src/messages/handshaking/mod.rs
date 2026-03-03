mod clientbound;
mod serverbound;

pub use self::serverbound::*;

pub enum ClientboundHandshakingMessage {}

pub enum ServerboundHandshakingMessage {
    Handshake(Handshake),
}
