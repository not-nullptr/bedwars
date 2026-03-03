use std::{collections::HashMap, sync::Arc};

use protocol::{
    GameProfile, Identifier, Readable, Writable,
    messages::{
        configuration::RegistryEntry,
        handshaking::Handshake,
        login::{ClientboundLoginMessage, LoginSuccess, ServerboundLoginMessage},
    },
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};

use crate::{
    config::Config,
    registry::Registry,
    server::{handlers::configuration::ConfigurationHandler, msg::ServerMessage},
};

pub struct LoginHandler<Io> {
    config: Arc<Config>,
    registry: Arc<Registry>,
    io: Io,
    handshake: Handshake,
    tx: mpsc::Sender<ServerMessage<Io>>,
}

impl<Io: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static> LoginHandler<Io> {
    pub fn new(
        io: Io,
        config: Arc<Config>,
        registry: Arc<Registry>,
        handshake: Handshake,
        tx: mpsc::Sender<ServerMessage<Io>>,
    ) -> Self {
        Self {
            config,
            io,
            registry,
            handshake,
            tx,
        }
    }

    pub async fn handle(mut self) -> color_eyre::Result<()> {
        loop {
            let ServerboundLoginMessage::LoginStart(info) =
                ServerboundLoginMessage::read_from(&mut self.io).await?
            else {
                tracing::warn!("unexpected login message");
                return Ok(());
            };

            let profile = GameProfile {
                name: info.name,
                id: info.id,
                properties: Vec::new(),
            };

            let login_success = ClientboundLoginMessage::LoginSuccess(LoginSuccess {
                profile: profile.clone(),
            });

            login_success.write_into(&mut self.io).await?;

            loop {
                match ServerboundLoginMessage::read_from(&mut self.io).await? {
                    ServerboundLoginMessage::LoginAcknowledged(_) => break,
                    _ => {
                        tracing::warn!("unexpected login message");
                        continue;
                    }
                }
            }

            let configuration = ConfigurationHandler::new(self.io, self.tx, self.registry, profile);
            configuration.handle().await?;
            return Ok(());
        }
    }
}
