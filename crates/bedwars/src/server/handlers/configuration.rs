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
        let mut ident_to_idx = HashMap::new();
        let mut update_tags = Vec::new();

        for data in self.registry.registry_data() {
            static ALLOWED: LazyLock<HashSet<Identifier>> = LazyLock::new(|| {
                HashSet::from([
                    Identifier::new("banner_pattern"),
                    Identifier::new("chat_type"),
                    Identifier::new("damage_type"),
                    Identifier::new("dimension_type"),
                    Identifier::new("instrument"),
                    Identifier::new("jukebox_song"),
                    Identifier::new("painting_variant"),
                    Identifier::new("test_environment"),
                    Identifier::new("test_instance"),
                    Identifier::new("timeline"),
                    Identifier::new("trim_material"),
                    Identifier::new("trim_pattern"),
                    Identifier::new("worldgen/biome"),
                    Identifier::new("cat_variant"),
                    Identifier::new("chicken_variant"),
                    Identifier::new("cow_variant"),
                    Identifier::new("frog_variant"),
                    Identifier::new("pig_variant"),
                    Identifier::new("wolf_variant"),
                    Identifier::new("wolf_sound_variant"),
                    Identifier::new("zombie_nautilus_variant"),
                ])
            });

            if !ALLOWED.contains(&data.registry_id) {
                continue;
            }

            let registry_data = ClientboundConfigurationMessage::RegistryData(RegistryData {
                registry_id: data.registry_id.clone(),
                entries: data
                    .entries
                    .iter()
                    .map(|e| {
                        ident_to_idx.insert(e.entry_id.clone(), ident_to_idx.len());
                        RegistryEntry {
                            entry_id: e.entry_id.clone(),
                            nbt: e.nbt.as_ref().map(|n| n.clone()),
                        }
                    })
                    .collect(),
            });

            if let Err(e) = registry_data.write_into(&mut self.io).await {
                // write it into a vec and print it as hex for debugging
                return Err(e.into());
            }

            update_tags.push(TaggedRegistry {
                registry: data.registry_id.clone(),
                tags: self
                    .registry
                    .tag_map()
                    .iter()
                    .map(|(k, v)| Tag {
                        tag_name: k.clone(),
                        entries: v
                            .iter()
                            .filter_map(|i| ident_to_idx.get(i).map(|idx| VarInt::new(*idx as u32)))
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
                    break;
                }
            }
        }

        self.tx
            .send(ServerMessage::Login(self.io, self.profile))
            .await?;

        Ok(())
    }
}
