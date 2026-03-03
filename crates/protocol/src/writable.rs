use crate::RwError;
use tokio::io::AsyncWrite;

pub trait Writable {
    fn write_into<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> impl Future<Output = Result<(), RwError>>;
}
