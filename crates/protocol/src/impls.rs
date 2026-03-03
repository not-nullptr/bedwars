use crate::{Readable, Writable, varint::VarInt};

macro_rules! num_impl {
    ($($ty:ty)*) => {
        $(
            impl $crate::Readable for $ty {
                async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
                    reader: &mut R,
                ) -> Result<Self, $crate::RwError> {
                    use ::tokio::io::AsyncReadExt;
                    let mut buf = [0; ::std::mem::size_of::<$ty>()];
                    reader.read_exact(&mut buf).await?;
                    Ok(Self::from_le_bytes(buf))
                }
            }

            impl $crate::Writable for $ty {
                async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
                    &self,
                    writer: &mut W,
                ) -> Result<(), crate::RwError> {
                    use ::tokio::io::AsyncWriteExt;
                    writer.write_all(&self.to_le_bytes()).await?;
                    Ok(())
                }
            }
        )*
    };
}

num_impl!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize);

impl Readable for bool {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let byte = u8::read_from(reader).await?;
        Ok(byte != 0)
    }
}

impl Writable for bool {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        (*self as u8).write_into(writer).await
    }
}

impl Readable for VarInt {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let mut num = 0u32;
        let mut shift = 0;
        loop {
            let byte = u8::read_from(reader).await?;
            num |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        Ok(Self::from_u32(num))
    }
}

impl Writable for VarInt {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let mut num = self.into_inner();
        loop {
            let mut byte = (num & 0x7F) as u8;
            num >>= 7;
            if num != 0 {
                byte |= 0x80;
            }
            byte.write_into(writer).await?;
            if num == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl Readable for String {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let bytes = Vec::<u8>::read_from(reader).await?;
        Ok(String::from_utf8(bytes)?)
    }
}

impl Writable for String {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        self.as_bytes().write_into(writer).await
    }
}

impl<T: Readable> Readable for Vec<T> {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let len = VarInt::read_from(reader).await?.into_inner() as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read_from(reader).await?);
        }
        Ok(vec)
    }
}

impl<T: Writable> Writable for Vec<T> {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        VarInt::from_u32(self.len() as u32)
            .write_into(writer)
            .await?;
        for item in self {
            item.write_into(writer).await?;
        }
        Ok(())
    }
}

impl<T: Readable, const N: usize> Readable for [T; N] {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let mut arr = std::mem::MaybeUninit::<[T; N]>::uninit();
        let ptr = arr.as_mut_ptr() as *mut T;
        for i in 0..N {
            unsafe {
                ptr.add(i).write(T::read_from(reader).await?);
            }
        }
        Ok(unsafe { arr.assume_init() })
    }
}

impl<T: Writable, const N: usize> Writable for [T; N] {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        for item in self {
            item.write_into(writer).await?;
        }
        Ok(())
    }
}

impl<'a, T: Writable> Writable for &'a [T] {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        VarInt::from_u32(self.len() as u32)
            .write_into(writer)
            .await?;
        for item in *self {
            item.write_into(writer).await?;
        }

        Ok(())
    }
}
