use std::sync::Arc;

use protocol::{
    Readable, Writable,
    json::Json,
    messages::{
        handshaking::Handshake,
        status::{
            ClientboundStatusMessage, Description, PlayerInfo, PongResponse,
            ServerboundStatusMessage, StatusData, StatusResponse, VersionInfo,
        },
    },
};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::config::Config;

pub struct StatusHandler<Io> {
    config: Arc<Config>,
    io: Io,
    handshake: Handshake,
}

impl<Io: AsyncRead + AsyncWrite + Unpin> StatusHandler<Io> {
    pub fn new(io: Io, config: Arc<Config>, handshake: Handshake) -> Self {
        Self {
            config,
            io,
            handshake,
        }
    }

    pub async fn handle(&mut self) -> color_eyre::Result<()> {
        loop {
            let msg = ServerboundStatusMessage::read_from(&mut self.io).await?;

            match msg {
                ServerboundStatusMessage::StatusRequest(_) => {
                    let response = ClientboundStatusMessage::StatusResponse(StatusResponse {
                        json_response: Json(StatusData {
                            version: VersionInfo {
                                name: "Bedwars".to_string(),
                                protocol: i32::from(self.handshake.protocol_version) as u32,
                            },
                            players: PlayerInfo {
                                online: 0,
                                max: self.config.server.max_players,
                                sample: Vec::new(),
                            },
                            description: Description {
                                text: self.config.server.motd.clone(),
                            },
                            enforces_secure_chat: false,
                            favicon: None,
                        }),
                    });

                    response.write_into(&mut self.io).await?;
                }
                ServerboundStatusMessage::PingRequest(p) => {
                    let response = ClientboundStatusMessage::PongResponse(PongResponse {
                        timestamp: p.timestamp,
                    });

                    response.write_into(&mut self.io).await?;
                }
            }
        }
    }
}
