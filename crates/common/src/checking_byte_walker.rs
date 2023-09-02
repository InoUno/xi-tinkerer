use std::cmp::min;

use crate::{
    byte_functions::HasByteFunctions, byte_walker::ByteWalker,
    writing_byte_walker::WritingByteWalker,
};

use anyhow::Result;

pub struct CheckingByteWalker<BW: ByteWalker> {
    original: BW,
}

impl<BW: ByteWalker> CheckingByteWalker<BW> {
    pub fn new(original: BW) -> Self {
        Self { original }
    }
}

impl<BW: ByteWalker> ByteWalker for CheckingByteWalker<BW> {
    fn goto_usize(&mut self, offset: usize) {
        self.original.goto_usize(offset);
    }

    fn skip(&mut self, count: usize) {
        self.original.skip(count);
    }

    fn offset(&self) -> usize {
        self.original.offset()
    }

    fn len(&self) -> usize {
        self.original.len()
    }

    fn read_bytes_at(&mut self, offset: usize, amount: usize) -> Result<&[u8]> {
        self.original.read_bytes_at(offset, amount)
    }

    fn take_bytes(&mut self, amount: usize) -> Result<&[u8]> {
        self.original.take_bytes(amount)
    }
}

impl<BW: ByteWalker> WritingByteWalker for CheckingByteWalker<BW> {
    fn write_bytes_at(&mut self, offset: usize, bytes: &[u8]) {
        assert_eq!(
            self.original.read_bytes_at(offset, bytes.len()).unwrap(),
            bytes
        );
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        let original_bytes = self.original.take_bytes(bytes.len()).unwrap();
        if bytes.len() != original_bytes.len() || bytes != original_bytes {
            let first_diff_idx = original_bytes
                .iter()
                .zip(bytes.iter())
                .enumerate()
                .find_map(|(idx, (original_byte, encoded_byte))| {
                    if original_byte != encoded_byte {
                        Some(idx)
                    } else {
                        None
                    }
                });

            if first_diff_idx.is_none() {
                panic!(
                    "Mismatched bytes lengths: {} vs {}",
                    original_bytes.len(),
                    bytes.len()
                );
            }
            let first_diff_idx = first_diff_idx.unwrap();

            let context_start = first_diff_idx.saturating_sub(10);
            let context_end = min(bytes.len(), min(original_bytes.len(), first_diff_idx + 10));

            panic!(
                "Mismatched bytes at index {}:\n{:02X?}\n{:02X?}",
                first_diff_idx,
                &original_bytes[context_start..context_end],
                &bytes[context_start..context_end],
            );
        }
    }

    fn write_be<T: HasByteFunctions + Eq + std::fmt::Debug>(&mut self, value: T) {
        assert_eq!(self.original.step_be::<T>().unwrap(), value);
    }

    fn write_le<T: HasByteFunctions + Eq + std::fmt::Debug>(&mut self, value: T) {
        assert_eq!(self.original.step_le::<T>().unwrap(), value);
    }

    fn write_be_at<T: HasByteFunctions + Eq + std::fmt::Debug>(&mut self, offset: usize, value: T) {
        assert_eq!(self.original.read_be_at::<T>(offset).unwrap(), value);
    }

    fn write_le_at<T: HasByteFunctions + Eq + std::fmt::Debug>(&mut self, offset: usize, value: T) {
        assert_eq!(self.original.read_le_at::<T>(offset).unwrap(), value);
    }

    fn set_size(&mut self, _size: usize) {}

    fn into_vec(mut self) -> Vec<u8> {
        self.original
            .read_bytes_at(0, self.original.len())
            .unwrap()
            .to_vec()
    }
}
