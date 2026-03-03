use protocol::{Readable, VarInt, messages::handshaking::Handshake};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    sync::mpsc,
};

use crate::server::msg::ServerMessage;

pub async fn handle_connection<Io: AsyncRead + AsyncWrite + Send + Sync + 'static + Unpin>(
    mut stream: Io,
    tx: mpsc::Sender<ServerMessage<Io>>,
) -> color_eyre::Result<()> {
    let length = VarInt::read_from(&mut stream).await?;

    let mut reader = (&mut stream).take(length.value() as u64);

    let packet_id = VarInt::read_from(&mut reader).await?.value();

    if packet_id != 0 {
        tracing::warn!(
            "Expected handshake packet (id 0), got packet id {}",
            packet_id
        );
        return Ok(());
    }

    let handshake = Handshake::read_from(&mut reader).await?;
    tracing::info!(?handshake, "received handshake packet");

    tx.send(ServerMessage::Connection(stream, handshake))
        .await?;

    Ok(())
}
