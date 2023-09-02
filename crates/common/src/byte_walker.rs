use std::{fmt::Display, str::from_utf8};

use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::{byte_functions::HasByteFunctions, expect};

#[derive(Error, Debug)]
pub enum ByteWalkerError {
    #[error("Trying to read buffer at index {requested_index:?}, but buffer is only {buffer_length:?} long.")]
    OutOfRange {
        buffer_length: usize,
        requested_index: usize,
    },
}

pub trait ByteWalker {
    fn goto(&mut self, offset: u32) {
        self.goto_usize(offset as usize);
    }

    fn goto_usize(&mut self, offset: usize);

    #[inline]
    fn goto_start(&mut self) {
        self.goto(0)
    }

    fn skip(&mut self, count: usize);

    fn offset(&self) -> usize;

    fn len(&self) -> usize;

    #[inline]
    fn remaining(&self) -> usize {
        self.len().saturating_sub(self.offset())
    }

    fn read_bytes_at(&mut self, offset: usize, amount: usize) -> Result<&[u8]>;

    fn read_bytes(&mut self, amount: usize) -> Result<&[u8]> {
        self.read_bytes_at(self.offset(), amount)
    }

    fn take_bytes(&mut self, amount: usize) -> Result<&[u8]>;

    #[inline]
    fn step_while(&mut self, condition: impl Fn(u8) -> bool) -> Result<&[u8]> {
        let start_offset = self.offset();
        let mut current_offset = start_offset;
        while let Ok(byte) = self.read_at::<u8>(current_offset) {
            if !condition(byte) {
                break;
            }
            current_offset += 1;
        }
        self.take_bytes(current_offset - start_offset)
    }

    #[inline]
    fn step_until(&mut self, end_char: u8) -> Result<&[u8]> {
        let start_offset = self.offset();
        let mut current_offset = start_offset;
        while let Ok(byte) = self.read_at::<u8>(current_offset) {
            if byte == end_char {
                break;
            }
            current_offset += 1;
        }
        self.take_bytes(current_offset - start_offset)
    }

    #[inline]
    fn step_until_chars<const N: usize>(&mut self, end_chars: [u8; N]) -> Result<&[u8]> {
        let start_offset = self.offset();
        let mut current_offset = start_offset;
        while let Ok(bytes) = self.read_bytes_at(current_offset, N) {
            if bytes == end_chars {
                break;
            }
            current_offset += 1;
        }
        self.take_bytes(current_offset - start_offset)
    }

    #[inline]
    fn step_until_escaped(&mut self, end_char: u8, escape_char: u8) -> Result<&[u8]> {
        let start_offset = self.offset();
        let mut current_offset = start_offset;
        while let Ok(byte) = self.read_at::<u8>(current_offset) {
            if byte == escape_char {
                current_offset += 1;
            } else if byte == end_char {
                break;
            }
            current_offset += 1;
        }
        self.take_bytes(current_offset - start_offset)
    }

    // Read functions
    fn read_be<T: HasByteFunctions>(&mut self) -> Result<T> {
        let bytes = self.read_bytes(std::mem::size_of::<T>())?;
        T::from_be_bytes(bytes)
    }

    fn read_le<T: HasByteFunctions>(&mut self) -> Result<T> {
        let bytes = self.read_bytes(std::mem::size_of::<T>())?;
        T::from_le_bytes(bytes)
    }

    fn read<T: HasByteFunctions>(&mut self) -> Result<T> {
        self.read_le::<T>()
    }

    fn read_be_at<T: HasByteFunctions>(&mut self, offset: usize) -> Result<T> {
        let bytes = self.read_bytes_at(offset, std::mem::size_of::<T>())?;
        T::from_be_bytes(bytes)
    }

    fn read_le_at<T: HasByteFunctions>(&mut self, offset: usize) -> Result<T> {
        let bytes = self.read_bytes_at(offset, std::mem::size_of::<T>())?;
        T::from_le_bytes(bytes)
    }

    fn read_at<T: HasByteFunctions>(&mut self, offset: usize) -> Result<T> {
        self.read_le_at::<T>(offset)
    }

    // Take functions
    fn step_be<T: HasByteFunctions>(&mut self) -> Result<T> {
        let bytes = self.take_bytes(std::mem::size_of::<T>())?;
        T::from_be_bytes(bytes)
    }

    fn step_le<T: HasByteFunctions>(&mut self) -> Result<T> {
        let bytes = self.take_bytes(std::mem::size_of::<T>())?;
        T::from_le_bytes(bytes)
    }

    fn step<T: HasByteFunctions>(&mut self) -> Result<T> {
        self.step_le::<T>()
    }

    fn expect<T: HasByteFunctions + Eq + Display>(&mut self, val: T) -> Result<()> {
        let read_val = self.step_le::<T>()?;

        let res = expect(val, read_val);
        if let Err(err) = res {
            return Err(anyhow!("At offset {}: {}", self.offset(), err));
        }
        res
    }

    fn expect_msg<T: HasByteFunctions + Eq + Display>(
        &mut self,
        val: T,
        message: impl AsRef<str>,
    ) -> Result<()> {
        let read_val = self.step_le::<T>()?;

        let res = expect(val, read_val);
        if let Err(err) = res {
            return Err(anyhow!("{}: {}", message.as_ref(), err));
        }
        res
    }

    fn expect_n_msg<T: HasByteFunctions + Eq + Display + Copy>(
        &mut self,
        val: T,
        amount: usize,
        message: impl AsRef<str>,
    ) -> Result<()> {
        for idx in 0..amount {
            let read_val = self.step_le::<T>()?;

            let res = expect(val, read_val);
            if let Err(err) = res {
                return Err(anyhow!("{} [index {}]: {}", message.as_ref(), idx, err));
            }
        }
        Ok(())
    }

    fn expect_utf8_str(&mut self, val: &str) -> Result<()> {
        let read_val = from_utf8(self.take_bytes(val.len())?)?;
        expect(val, read_val)
    }
}

pub struct BufferedByteWalker<T> {
    pub(crate) data: T,
    pub(crate) offset: usize,
}

impl<T> BufferedByteWalker<T> {
    pub fn on(buffer: T) -> Self {
        Self {
            data: buffer,
            offset: 0,
        }
    }

    pub fn goto(&mut self, offset: u32) {
        self.offset = offset as usize;
    }

    pub fn rewind(&mut self) {
        self.offset = 0;
    }
}

impl<S> ByteWalker for BufferedByteWalker<S>
where
    S: AsRef<[u8]>,
{
    fn goto_usize(&mut self, offset: usize) {
        self.offset = offset;
    }

    fn skip(&mut self, count: usize) {
        self.offset += count;
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn len(&self) -> usize {
        self.data.as_ref().len()
    }

    fn read_bytes_at(&mut self, offset: usize, amount: usize) -> Result<&[u8]> {
        let end_offset = offset + amount;
        if end_offset > self.data.as_ref().len() {
            return Err(ByteWalkerError::OutOfRange {
                buffer_length: self.data.as_ref().len(),
                requested_index: end_offset,
            }
            .into());
        }

        Ok(&self.data.as_ref()[offset..end_offset])
    }

    fn take_bytes(&mut self, amount: usize) -> Result<&[u8]> {
        self.skip(amount);
        self.read_bytes_at(self.offset() - amount, amount)
    }
}

impl<S> BufferedByteWalker<S>
where
    S: AsRef<[u8]>,
{
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }
}

impl<S> BufferedByteWalker<S>
where
    S: AsMut<[u8]>,
{
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::writing_byte_walker::WritingByteWalker;

    use super::{BufferedByteWalker, ByteWalker};

    #[test]
    fn simple_walk() {
        let data = vec![127, 0, 0, 1];
        let mut bytes = BufferedByteWalker::on(&data[..]);

        assert_eq!(bytes.step::<u8>().unwrap(), 127);
        assert_eq!(bytes.step::<u8>().unwrap(), 0);
        assert_eq!(bytes.step::<u8>().unwrap(), 0);
        assert_eq!(bytes.step::<u8>().unwrap(), 1);
        assert!(bytes.step::<u8>().is_err());

        bytes.rewind();
        assert_eq!(bytes.step::<u32>().unwrap(), 16777343);
    }

    #[test]
    fn writing() {
        let mut bytes = BufferedByteWalker::with_size(4);

        bytes.write(127u8);
        bytes.write(0u8);
        bytes.write(0u8);
        bytes.write(1u8);

        assert_eq!(bytes.data.len(), 4);
        assert_eq!(bytes.data[0], 127);
        assert_eq!(bytes.data[1], 0);
        assert_eq!(bytes.data[2], 0);
        assert_eq!(bytes.data[3], 1);
    }

    #[test]
    fn expanding() {
        let mut bytes = BufferedByteWalker::new();

        bytes.write(127u8);
        bytes.write(0u8);
        bytes.write(0u8);
        bytes.write(1u8);

        assert_eq!(bytes.data.len(), 4);
        assert_eq!(bytes.data[0], 127);
        assert_eq!(bytes.data[1], 0);
        assert_eq!(bytes.data[2], 0);
        assert_eq!(bytes.data[3], 1);
    }

    #[test]
    fn expanding_u32() {
        let mut bytes = BufferedByteWalker::new();

        bytes.write(16777343u32);

        assert_eq!(bytes.data.len(), 4);
        assert_eq!(bytes.data[0], 127);
        assert_eq!(bytes.data[1], 0);
        assert_eq!(bytes.data[2], 0);
        assert_eq!(bytes.data[3], 1);
    }

    #[test]
    fn expanding_bytes() {
        let mut bytes = BufferedByteWalker::new();

        bytes.write_bytes(&[127, 0, 0, 1]);

        assert_eq!(bytes.data.len(), 4);
        assert_eq!(bytes.data[0], 127);
        assert_eq!(bytes.data[1], 0);
        assert_eq!(bytes.data[2], 0);
        assert_eq!(bytes.data[3], 1);
    }
}
