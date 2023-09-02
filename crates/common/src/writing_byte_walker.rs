use crate::{byte_functions::HasByteFunctions, byte_walker::ByteWalker};

pub trait WritingByteWalker: ByteWalker {
    fn write_bytes_at(&mut self, offset: usize, bytes: &[u8]);
    fn write_bytes(&mut self, bytes: &[u8]);

    fn write_str(&mut self, str: &str) {
        self.write_bytes(str.as_bytes());
    }

    fn write_be<T: HasByteFunctions>(&mut self, value: T);
    fn write_le<T: HasByteFunctions>(&mut self, value: T);
    fn write<T: HasByteFunctions>(&mut self, value: T) {
        self.write_le::<T>(value);
    }

    fn write_be_at<T: HasByteFunctions>(&mut self, offset: usize, value: T);
    fn write_le_at<T: HasByteFunctions>(&mut self, offset: usize, value: T);
    fn write_at<T: HasByteFunctions>(&mut self, offset: usize, value: T) {
        self.write_le_at::<T>(offset, value);
    }

    fn set_size(&mut self, size: usize);

    fn into_vec(self) -> Vec<u8>;
}
