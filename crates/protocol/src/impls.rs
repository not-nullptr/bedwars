use fastnbt::{DeOpts, SerOpts};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use crate::{Identifier, Position, Readable, Writable, json::Json, varint::VarInt};

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
                    Ok(Self::from_be_bytes(buf))
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

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

impl Readable for VarInt {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let mut num = 0;
        let mut shift = 0;
        loop {
            let byte = u8::read_from(reader).await?;
            num |= ((byte & SEGMENT_BITS) as i32) << shift;
            if byte & CONTINUE_BIT == 0 {
                break;
            }
            shift += 7;
        }
        Ok(Self::from(num))
    }
}

impl Writable for VarInt {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let mut num = self.value();
        loop {
            let mut byte = (num & SEGMENT_BITS as i32) as u8;
            num >>= 7;
            if num != 0 {
                byte |= CONTINUE_BIT;
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
        let len = crate::varint::VarInt::read_from(reader).await?.value() as usize;
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
        VarInt::from(self.len() as i32).write_into(writer).await?;
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
        VarInt::from(self.len() as i32).write_into(writer).await?;
        for item in *self {
            item.write_into(writer).await?;
        }

        Ok(())
    }
}

impl<T: serde::Serialize> Writable for Json<T> {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let json_str = serde_json::to_string(&self.0)?;
        json_str.write_into(writer).await?;
        Ok(())
    }
}

impl<T: serde::de::DeserializeOwned> Readable for Json<T> {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let json_str = String::read_from(reader).await?;
        let value = serde_json::from_str(&json_str)?;
        Ok(Json(value))
    }
}

impl<T: Readable> Readable for Option<T> {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let present = bool::read_from(reader).await?;
        if present {
            Ok(Some(T::read_from(reader).await?))
        } else {
            Ok(None)
        }
    }
}

impl<T: Writable> Writable for Option<T> {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        match self {
            Some(value) => {
                true.write_into(writer).await?;
                value.write_into(writer).await?;
            }
            None => {
                false.write_into(writer).await?;
            }
        }
        Ok(())
    }
}

impl Readable for Uuid {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf).await?;
        Ok(Uuid::from_bytes(buf))
    }
}

impl Writable for Uuid {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        writer.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

impl Readable for Position {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let val = u64::read_from(reader).await?;
        #[inline]
        fn sign_extend(val: u32, bits: u32) -> i32 {
            ((val << (32 - bits)) as i32) >> (32 - bits)
        }

        let x = sign_extend(((val >> 38) & 0x3FF_FFFF) as u32, 26);
        let z = sign_extend(((val >> 12) & 0x3FF_FFFF) as u32, 26);
        let y = sign_extend((val & 0xFFF) as u32, 12) as i16;

        Ok(Position::new(x, y, z))
    }
}

impl Writable for Position {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let x = (self.x as i64 & 0x03FF_FFFF) as u64;
        let z = (self.z as i64 & 0x03FF_FFFF) as u64;
        let y = (self.y as i32 & 0x0FFF) as u64;

        let val = (x << 38) | (z << 12) | y;
        val.write_into(writer).await?;

        Ok(())
    }
}

impl<'a, T: Writable> Writable for &'a T {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        (*self).write_into(writer).await
    }
}

impl Readable for Identifier {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let s = String::read_from(reader).await?;
        let mut parts = s.splitn(2, ':');
        let namespace = parts.next().unwrap_or("").to_string();
        let value = parts.next().unwrap_or("").to_string();
        Ok(Identifier::with_namespace(namespace, value))
    }
}

impl Writable for Identifier {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let s = format!("{}:{}", self.namespace, self.value);
        s.write_into(writer).await?;
        Ok(())
    }
}

impl Readable for () {
    async fn read_from<R: ::tokio::io::AsyncRead + Unpin>(
        _reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        Ok(())
    }
}

impl Writable for () {
    async fn write_into<W: ::tokio::io::AsyncWrite + Unpin>(
        &self,
        _writer: &mut W,
    ) -> Result<(), crate::RwError> {
        Ok(())
    }
}

impl Writable for fastnbt::Value {
    async fn write_into<W: tokio::io::AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::RwError> {
        let bytes = fastnbt::to_bytes_with_opts(self, SerOpts::network_nbt())?;
        writer.write_all(&bytes).await?;
        Ok(())
    }
}

impl Readable for fastnbt::Value {
    async fn read_from<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, crate::RwError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let value = fastnbt::from_bytes_with_opts(&bytes, DeOpts::network_nbt())?;
        Ok(value)
    }
}
