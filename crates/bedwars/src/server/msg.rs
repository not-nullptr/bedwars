use protocol::{GameProfile, messages::handshaking::Handshake};

pub enum ServerMessage<Io> {
    Connection(Io, Handshake),
    Login(Io, GameProfile),
}
