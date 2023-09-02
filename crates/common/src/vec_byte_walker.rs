use crate::{
    byte_functions::HasByteFunctions, byte_walker::BufferedByteWalker,
    writing_byte_walker::WritingByteWalker,
};

pub type VecByteWalker = BufferedByteWalker<Vec<u8>>;

impl VecByteWalker {
    pub fn new() -> Self {
        Self {
            data: vec![],
            offset: 0,
        }
    }

    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
            offset: 0,
        }
    }
}

impl WritingByteWalker for VecByteWalker {
    fn write_bytes_at(&mut self, offset: usize, bytes: &[u8]) {
        let end = offset + bytes.len();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }

        self.data[offset..end].copy_from_slice(bytes);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_bytes_at(self.offset, bytes);
        self.offset += bytes.len();
    }

    fn write_be<T: HasByteFunctions>(&mut self, value: T) {
        let end = self.offset + std::mem::size_of::<T>();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }

        value.insert_into_be(&mut self.data[self.offset..end]);
        self.offset = end;
    }

    fn write_le<T: HasByteFunctions>(&mut self, value: T) {
        let end = self.offset + std::mem::size_of::<T>();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }

        value.insert_into_le(&mut self.data[self.offset..end]);
        self.offset = end;
    }

    fn write_be_at<T: HasByteFunctions>(&mut self, offset: usize, value: T) {
        let end = offset + std::mem::size_of::<T>();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }

        value.insert_into_be(&mut self.data[offset..end]);
    }

    fn write_le_at<T: HasByteFunctions>(&mut self, offset: usize, value: T) {
        let end = offset + std::mem::size_of::<T>();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }

        value.insert_into_le(&mut self.data[offset..end]);
    }

    fn set_size(&mut self, size: usize) {
        self.data.resize(size, 0);
    }

    fn into_vec(self) -> Vec<u8> {
        self.data
    }
}
