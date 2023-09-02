use anyhow::Result;
use std::{
    cmp::min,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::byte_walker::{ByteWalker, ByteWalkerError};

pub struct FileByteWalker {
    file: File,
    offset: usize,
    temp_buffer: Vec<u8>,
}

impl FileByteWalker {
    pub fn new(file: File) -> Self {
        Self {
            file,
            offset: 0,
            temp_buffer: Vec::with_capacity(4),
        }
    }

    pub fn from_path(path: &PathBuf) -> Result<Self> {
        Ok(Self {
            file: File::open(path)?,
            offset: 0,
            temp_buffer: Vec::with_capacity(4),
        })
    }
}

impl ByteWalker for FileByteWalker {
    fn goto_usize(&mut self, offset: usize) {
        let _ = self.file.seek(SeekFrom::Start(offset as u64));
        self.offset = offset;
    }

    fn skip(&mut self, count: usize) {
        let _ = self.file.seek(SeekFrom::Current(count as i64));
        self.offset += count;
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn len(&self) -> usize {
        self.file
            .metadata()
            .map(|metadata| metadata.len())
            .unwrap_or_default() as usize
    }

    fn read_bytes_at(&mut self, offset: usize, amount: usize) -> Result<&[u8]> {
        let saved_offset = self.offset();
        self.goto_usize(offset);

        let len = self.len();
        if len <= offset {
            return Err(ByteWalkerError::OutOfRange {
                buffer_length: len,
                requested_index: offset,
            }
            .into());
        }
        let amount = min(len - offset, amount);

        if self.temp_buffer.len() < amount {
            self.temp_buffer.resize(amount, 0);
        }
        self.file.read_exact(&mut self.temp_buffer[..amount])?;

        self.goto_usize(saved_offset);

        Ok(&self.temp_buffer[..amount])
    }

    fn take_bytes(&mut self, amount: usize) -> Result<&[u8]> {
        let len = self.len();
        let amount = min(len - self.offset, amount);

        if self.temp_buffer.len() < amount {
            self.temp_buffer.resize(amount, 0);
        }

        self.file.read_exact(&mut self.temp_buffer[..amount])?;
        self.offset += amount;

        Ok(&self.temp_buffer[..amount])
    }
}
