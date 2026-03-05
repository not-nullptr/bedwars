mod connection;
mod handlers;
pub mod msg;
mod state;
mod world;

use crate::{
    config::Config,
    entity::{Entity, EntityData, allocator::EntityAllocator, player::PlayerEntity},
    registry::Registry,
    server::{connection::Connection, msg::ServerMessage, world::World},
};
use generated::{Block, bedrock};
use protocol::{
    BitSet, Chunk, ChunkData, ChunkSection, Gamemode, HasData, HasDataKind, Identifier,
    PaletteFormat, PaletteFormatKind, PalettedContainer, TeleportFlags, VarInt, Writable,
    messages::{
        configuration::{RegistryData, RegistryEntry},
        play::{
            ChunkDataAndUpdateLight, ClientboundPlayMessage, GameEvent, LightData, Login,
            SetCenterChunk, SynchronizePlayerPosition,
        },
    },
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::mpsc,
    time::Interval,
};

pub struct Server<Io> {
    tx: mpsc::Sender<ServerMessage<Io>>,
    rx: mpsc::Receiver<ServerMessage<Io>>,
    tick: Interval,
    config: Arc<Config>,
    needs_onboarding: Vec<Connection<Io>>,
    clients: Vec<Connection<Io>>,
    entities: HashMap<i32, Entity>,
    allocator: EntityAllocator,
    registry: Arc<Registry>,
    world: World,
}

impl<Io: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static> Server<Io> {
    pub fn new(
        tx: mpsc::Sender<ServerMessage<Io>>,
        rx: mpsc::Receiver<ServerMessage<Io>>,
        config: Arc<Config>,
        registry: Arc<Registry>,
    ) -> Self {
        Self {
            tick: tokio::time::interval(Duration::from_millis(50)), // 20 tps
            tx,
            rx,
            registry,
            config,
            needs_onboarding: Vec::new(),
            clients: Vec::new(),
            allocator: EntityAllocator::new(),
            entities: HashMap::new(),
            world: World::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    if let Err(e) = self.handle_message(msg).await {
                        tracing::error!(%e, "error handling message");
                    }
                }

                _ = self.tick.tick() => {
                    if let Err(e) = self.tick().await {
                        tracing::error!(%e, "error ticking server");
                    }
                }
            }
        }
    }

    async fn tick(&mut self) -> color_eyre::Result<()> {
        self.onboard_available_connections().await?;
        Ok(())
    }

    async fn onboard_available_connections(&mut self) -> color_eyre::Result<()> {
        for conn in std::mem::take(&mut self.needs_onboarding) {
            if let Err(e) = self.onboard(conn).await {
                tracing::error!(%e, "error onboarding connection");
            }
        }

        Ok(())
    }

    async fn onboard(&mut self, mut conn: Connection<Io>) -> color_eyre::Result<()> {
        let entity = Entity::new(&mut self.allocator, EntityData::Player(PlayerEntity::new()));
        let id = entity.id;
        self.entities.insert(id, entity);

        let dimension = self
            .registry
            .reverse_sorted_registry()
            .get(&Identifier::const_new("dimension_type"))
            .and_then(|m| m.get(&Identifier::const_new("overworld")))
            .copied()
            .unwrap();

        let login = ClientboundPlayMessage::Login(Login {
            entity_id: id,
            is_hardcore: false,
            dimension_names: vec!["minecraft:overworld".to_string()],
            max_players: 100.into(),
            view_distance: 10.into(),
            simulation_distance: 10.into(),
            reduced_debug_info: false,
            enable_respawn_screen: true,
            do_limited_crafting: false,
            dimension_type: VarInt::new(dimension as i32),
            dimension_name: Identifier::with_namespace("minecraft", "overworld"),
            hashed_seed: 0,
            game_mode: Gamemode::Creative,
            previous_game_mode: 0,
            is_debug: false,
            is_flat: false,
            death_info: None,
            portal_cooldown: 0.into(),
            sea_level: 0.into(),
            enforces_secure_chat: false,
        });

        login.write_into(&mut conn.io).await?;

        let wait_chunks = ClientboundPlayMessage::GameEvent(GameEvent::StartWaitingForLevelChunks);
        wait_chunks.write_into(&mut conn.io).await?;

        let sync = ClientboundPlayMessage::SynchronizePlayerPosition(SynchronizePlayerPosition {
            teleport_id: VarInt::new(1),
            x: 0.0,
            y: 64.0,
            z: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            flags: TeleportFlags::empty(),
        });

        sync.write_into(&mut conn.io).await?;

        let chunk_center = ClientboundPlayMessage::SetCenterChunk(SetCenterChunk {
            chunk_x: VarInt::new(0),
            chunk_z: VarInt::new(0),
        });

        chunk_center.write_into(&mut conn.io).await?;

        let bedrock = Block::Piston(generated::piston::Piston {
            extended: generated::piston::Extended::True,
            facing: generated::piston::Facing::Down,
        });

        for x in -1..=1 {
            for y in -1..=1 {
                let chunk = self.world.chunks.get(&(x, y)).unwrap();
                let chunks =
                    ClientboundPlayMessage::ChunkDataAndUpdateLight(ChunkDataAndUpdateLight {
                        chunk_x: x,
                        chunk_z: y,
                        data: ChunkData {
                            block_entities: vec![],
                            heightmaps: vec![],
                            data: chunk.clone(),
                        },
                        light: LightData {
                            block_light_arrays: vec![],
                            block_light_mask: BitSet::empty(),
                            empty_block_light_mask: BitSet::empty(),
                            empty_sky_light_mask: BitSet::empty(),
                            sky_light_arrays: vec![],
                            sky_light_mask: BitSet::empty(),
                        },
                    });

                chunks.write_into(&mut conn.io).await?;
            }
        }

        // for x in -1..=1 {
        //     for z in -1..=1 {
        //         let chunk = self.world.chunks.get(&(x, z)).unwrap();
        //         let chunk_data =
        //             ClientboundPlayMessage::ChunkDataAndUpdateLight(ChunkDataAndUpdateLight {
        //                 chunk_x: x,
        //                 chunk_z: z,
        //                 data: ChunkData {
        //                     block_entities: vec![],
        //                     heightmaps: vec![],
        //                     data: chunk.clone(),
        //                 },
        //                 light: LightData {
        //                     block_light_arrays: vec![],
        //                     block_light_mask: BitSet::empty(),
        //                     empty_block_light_mask: BitSet::empty(),
        //                     empty_sky_light_mask: BitSet::empty(),
        //                     sky_light_arrays: vec![],
        //                     sky_light_mask: BitSet::empty(),
        //                 },
        //             });

        //         chunk_data.write_into(&mut conn.io).await?;
        //     }
        // }

        sync.write_into(&mut conn.io).await?;

        self.clients.push(conn);

        Ok(())
    }

    async fn handle_message(&mut self, msg: ServerMessage<Io>) -> color_eyre::Result<()> {
        match msg {
            ServerMessage::Connection(stream, handshake) => {
                tracing::info!(?handshake, "handling new connection");
                let config = self.config.clone();
                let registry = self.registry.clone();
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handlers::handle(stream, handshake, config, registry, tx).await
                    {
                        tracing::error!(%e, "error handling connection");
                    }
                });
            }

            ServerMessage::Login(io, profile) => {
                tracing::info!(?profile, "handling login");
                let connection = Connection { io, profile };
                self.needs_onboarding.push(connection);
            }
        }

        Ok(())
    }
}
