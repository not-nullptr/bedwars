use crate::RwError;
use tokio::io::AsyncRead;

pub trait Readable: Sized {
    fn read_from<R: AsyncRead + Unpin>(
        reader: &mut R,
    ) -> impl Future<Output = Result<Self, RwError>>;
}
