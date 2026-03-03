use crate::{Readable, VarInt, Writable};

#[derive(Debug, Clone, Readable, Writable)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub intent: Intent,
}

#[derive(Debug, Clone, Copy, Readable, Writable)]
#[net_repr(VarInt)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
}
