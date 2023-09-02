use anyhow::Result;

pub trait HasByteFunctions: Sized + Eq + std::fmt::Debug {
    fn from_be_bytes(bytes: &[u8]) -> Result<Self>;
    fn from_le_bytes(bytes: &[u8]) -> Result<Self>;
    fn insert_into_be(self, bytes: &mut [u8]);
    fn insert_into_le(self, bytes: &mut [u8]);
}

macro_rules! make_byte_functions {
    ($byte_type:ty) => {
        impl HasByteFunctions for $byte_type {
            fn from_be_bytes(bytes: &[u8]) -> Result<Self> {
                Ok(<$byte_type>::from_be_bytes(bytes.try_into()?))
            }

            fn from_le_bytes(bytes: &[u8]) -> Result<Self> {
                Ok(<$byte_type>::from_le_bytes(bytes.try_into()?))
            }

            fn insert_into_be(self, bytes: &mut [u8]) {
                bytes[..].copy_from_slice(&<$byte_type>::to_be_bytes(self));
            }

            fn insert_into_le(self, bytes: &mut [u8]) {
                bytes[..].copy_from_slice(&<$byte_type>::to_le_bytes(self));
            }
        }
    };
}

make_byte_functions!(u16);
make_byte_functions!(u32);
make_byte_functions!(u64);

make_byte_functions!(i16);
make_byte_functions!(i32);
make_byte_functions!(i64);

impl HasByteFunctions for u8 {
    fn from_be_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bytes[0])
    }

    fn from_le_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bytes[0])
    }

    fn insert_into_be(self, bytes: &mut [u8]) {
        bytes[0] = self;
    }

    fn insert_into_le(self, bytes: &mut [u8]) {
        bytes[0] = self
    }
}
