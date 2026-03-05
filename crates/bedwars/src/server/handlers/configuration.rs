use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock},
};

use crate::{registry::Registry, server::msg::ServerMessage};
use protocol::{
    GameProfile, Identifier, Readable, RwError, VarInt, Writable,
    messages::configuration::{
        ClientboundConfigurationMessage, FinishConfiguration, RegistryData, RegistryEntry,
        ServerboundConfigurationMessage, Tag, TaggedRegistry, UpdateTags,
    },
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
};

pub struct ConfigurationHandler<Io> {
    io: Io,
    tx: mpsc::Sender<ServerMessage<Io>>,
    registry: Arc<Registry>,
    profile: GameProfile,
}

impl<Io: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static> ConfigurationHandler<Io> {
    pub fn new(
        io: Io,
        tx: mpsc::Sender<ServerMessage<Io>>,
        registry: Arc<Registry>,
        profile: GameProfile,
    ) -> Self {
        Self {
            io,
            tx,
            registry,
            profile,
        }
    }

    pub async fn handle(mut self) -> color_eyre::Result<()> {
        let mut update_tags = Vec::new();

        // TODO: completely unnecessary cloning
        for data in self.registry.sorted_registry().iter() {
            let registry_data = ClientboundConfigurationMessage::RegistryData(RegistryData {
                registry_id: (*data.registry_id).clone(),
                entries: data
                    .entries
                    .iter()
                    .map(|entry| {
                        // ident_to_idx.insert(entry.entry_id.clone(), ident_to_idx.len());
                        RegistryEntry {
                            entry_id: (*entry.entry_id).clone(),
                            nbt: entry.nbt.clone().map(|v| (*v).clone()),
                        }
                    })
                    .collect(),
            });

            if let Err(e) = registry_data.write_into(&mut self.io).await {
                // write it into a vec and print it as hex for debugging
                return Err(e.into());
            }

            update_tags.push(TaggedRegistry {
                registry: (*data.registry_id).clone(),
                tags: self
                    .registry
                    .tag_map()
                    .iter()
                    .map(|(k, v)| Tag {
                        tag_name: (**k).clone(),
                        entries: v
                            .iter()
                            .filter_map(|i| {
                                self.registry
                                    .reverse_sorted_registry()
                                    .get(k)
                                    .and_then(|m| m.get(i))
                                    .map(|idx| VarInt::new(*idx as i32))
                            })
                            .collect(),
                    })
                    .collect(),
            });
        }

        let update_tags_msg = ClientboundConfigurationMessage::UpdateTags(UpdateTags {
            tagged_registries: update_tags,
        });

        if let Err(e) = update_tags_msg.write_into(&mut self.io).await {
            return Err(e.into());
        }

        tracing::debug!("finished sending registry data, waiting for client to acknowledge");

        let finish_configuration =
            ClientboundConfigurationMessage::FinishConfiguration(FinishConfiguration {});

        finish_configuration.write_into(&mut self.io).await?;

        loop {
            let msg = ServerboundConfigurationMessage::read_from(&mut self.io).await;

            match msg {
                Ok(ServerboundConfigurationMessage::AcknowledgeFinishConfiguration(_)) => break,

                Ok(ServerboundConfigurationMessage::ServerboundKnownPacks(_)) => {
                    todo!()
                }

                Err(RwError::InvalidEnumDiscriminant(n)) => {
                    tracing::warn!(%n, "invalid enum discriminant while waiting for AcknowledgeFinishConfiguration");
                }

                Err(e) => {
                    tracing::error!(%e, "error reading configuration message");
                    continue;
                }
            }
        }

        self.tx
            .send(ServerMessage::Login(self.io, self.profile))
            .await?;

        Ok(())
    }
}
