mod handler;
mod server;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // init tracing subscriber with level bedwars=debug,protocol=debug
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "bedwars=debug,protocol=debug",
        ))
        .init();

    tracing::info!("hello world!");

    Ok(())
}
