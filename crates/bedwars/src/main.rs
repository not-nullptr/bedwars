use crate::{config::Config, registry::Registry, server::Server};
use generated::{Block, bedrock::Bedrock};
use protocol::{Chunk, Identifier};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;

mod config;
mod entity;
mod handler;
mod registry;
mod server;
mod tag;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // init tracing subscriber with level bedwars=debug,protocol=debug
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "bedwars=debug,protocol=debug",
        ))
        .init();

    let config = Arc::new(Config::load("config.toml")?);

    let registry = Arc::new(
        Registry::discover(
            Path::new(&config.generated.path).join("data"),
            registry::NECESSARY_REGISTRIES,
        )
        .await?,
    );

    let listener = TcpListener::bind(&config.network.bind_address).await?;
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let mut server = Server::new(tx.clone(), rx, config, registry);

    tokio::spawn(async move {
        while let Ok(stream) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = handler::handle_connection(stream.0, tx).await {
                    tracing::error!(%e, "error handling connection");
                }
            });
        }
    });

    server.run().await;

    Ok(())
}
