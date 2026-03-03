mod configuration;
mod login;
mod status;

use crate::{
    config::Config,
    registry::Registry,
    server::{
        handlers::{login::LoginHandler, status::StatusHandler},
        msg::ServerMessage,
    },
};
use protocol::{
    Identifier,
    messages::{
        configuration::{RegistryData, RegistryEntry},
        handshaking::{Handshake, Intent},
    },
};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};

pub async fn handle<Io: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static>(
    io: Io,
    handshake: Handshake,
    config: Arc<Config>,
    registry: Arc<Registry>,
    tx: mpsc::Sender<ServerMessage<Io>>,
) -> color_eyre::Result<()> {
    match handshake.intent {
        Intent::Status => {
            let mut handler = StatusHandler::new(io, config, handshake);
            handler.handle().await?;
        }

        Intent::Login | Intent::Transfer => {
            let handler = LoginHandler::new(io, config, registry, handshake, tx);
            handler.handle().await?;
        }
    }

    Ok(())
}
