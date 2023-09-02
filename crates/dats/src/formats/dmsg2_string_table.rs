use std::{cmp::min, collections::BTreeMap};

use anyhow::{anyhow, Result};
use common::{
    byte_walker::ByteWalker, expect, get_padding, writing_byte_walker::WritingByteWalker,
};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

#[derive(Debug)]
struct Dmsg2StringTableHeaders {
    metadata_bytes: u32,
    string_entry_bytes: u32,
    list_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dmsg2StringTable {
    pub lists: BTreeMap<u32, Dmsg2StringList>,
}

#[derive(Debug)]
struct Dmsg2StringListMetadata {
    list_offset: u32,
    list_length: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dmsg2StringList {
    pub content: Vec<Dmsg2Content>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dmsg2Content {
    String { string: String },
    Flags { flags: Vec<u32> },
}

const HEADER_SIZE: u32 = 0x40;
const LIST_METADATA_SIZE: u32 = 0x08;
const LIST_STRING_PADDING: u32 = 28;

const MASK_U32: u32 = u32::MAX;
const MASK_U8: u8 = u8::MAX;

impl Dmsg2StringList {
    fn parse<T: ByteWalker>(
        walker: &mut T,
        list_metadata: Dmsg2StringListMetadata,
    ) -> Result<Dmsg2StringList> {
        let list_start_offset = walker.offset() as u32;

        let string_count: u32 = !walker.step()?;

        let string_metas = (0..string_count)
            .into_iter()
            .map(|_| {
                let string_offset = walker.step::<u32>()? ^ MASK_U32;
                let string_flags = walker.step::<u32>()? ^ MASK_U32;
                if string_offset + LIST_STRING_PADDING + 4 > list_metadata.list_length {
                    return Err(anyhow!(
                        "Invalid offset ({}) or flags ({}) for list length {}.",
                        string_offset,
                        string_flags,
                        list_metadata.list_length
                    ));
                }

                Ok((string_offset, string_flags))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut table_entry = Dmsg2StringList {
            content: Vec::with_capacity(string_count as usize),
        };

        // Read the actual strings
        for (idx, (string_offset, string_flags)) in string_metas.into_iter().enumerate() {
            // Validate we're at the expected string offset based on metadata
            let current_string_offset = walker.offset() as u32 - list_start_offset;

            if string_flags > 0 {
                // Parse as flags instead of string content.
                let flags = walker
                    .take_bytes(string_flags as usize * 4)?
                    .chunks(4)
                    .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) ^ MASK_U32)
                    .collect::<Vec<_>>();
                table_entry.content.push(Dmsg2Content::Flags { flags });
                continue;
            }

            if string_offset != current_string_offset {
                let bytes_inbetween = walker
                    .read_bytes_at(
                        (list_start_offset + min(string_offset, current_string_offset)) as usize,
                        (current_string_offset.abs_diff(string_offset)) as usize,
                    )?
                    .iter()
                    .map(|b| b ^ MASK_U8)
                    .collect::<Vec<_>>();

                return Err(anyhow!(
                    "Invalid string offset for {}: {} vs {}\nBytes in-between: {:02X?}",
                    idx,
                    string_offset,
                    current_string_offset,
                    bytes_inbetween
                ));
            }

            // Seemingly just padding with a 1 at the start.
            walker.expect_msg::<u8>(0x01 ^ MASK_U8, "Indication of string start")?;
            walker.expect_n_msg::<u8>(
                MASK_U8,
                LIST_STRING_PADDING as usize - 1,
                "Zero-padding before string",
            )?;

            // Read the string ending at a 0x00 (flipped to 0xFF in MASK_U8)
            let text_bytes: Vec<u8> = walker
                .step_until(MASK_U8)?
                .into_iter()
                .map(|byte| byte ^ MASK_U8)
                .collect();

            let string = Decoder::decode_simple(&text_bytes)?;
            walker.expect_msg::<u8>(MASK_U8, "End of string")?;

            // Alignment padding
            let padding = get_padding(text_bytes.len() + 1);
            walker.expect_n_msg::<u8>(MASK_U8, padding, "Alignment padding")?;

            table_entry.content.push(Dmsg2Content::String { string });
        }

        Ok(table_entry)
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<Dmsg2StringListMetadata> {
        let list_offset = walker.offset() as u32;

        // String count
        walker.write(!self.content.len() as i32);

        let encoded_strings = self
            .content
            .iter()
            .map(|entry| {
                match entry {
                    Dmsg2Content::String { string } => {
                        let mut bytes = Encoder::encode_simple(&string)?;
                        bytes.push(0x00); // End of string

                        // Add padding
                        let padding = get_padding(bytes.len());
                        for _ in 0..padding {
                            bytes.push(0x00);
                        }

                        // Flip the bytes
                        bytes.iter_mut().for_each(|byte| *byte ^= u8::MAX);

                        Ok((0u32, bytes))
                    }
                    Dmsg2Content::Flags { flags } => Ok((
                        flags.len() as u32,
                        flags
                            .iter()
                            .map(|int| int.to_le_bytes())
                            .flatten()
                            .collect(),
                    )),
                }
            })
            .collect::<Result<Vec<_>>>()?;

        // Write the string offsets and flags
        let metadata_len = self.content.len() as u32 * 8 + 4;

        let mut current_string_offset = metadata_len;
        for (flag, encoded_string) in &encoded_strings {
            walker.write::<u32>(current_string_offset ^ MASK_U32);
            walker.write::<u32>(flag ^ MASK_U32);

            current_string_offset += encoded_string.len() as u32 + LIST_STRING_PADDING;
        }

        // Write the strings
        for (_, encoded_string) in &encoded_strings {
            // String prefix
            walker.write(0x01 ^ MASK_U8);
            for _ in 0..LIST_STRING_PADDING - 1 {
                walker.write(MASK_U8);
            }

            walker.write_bytes(&encoded_string);
        }

        Ok(Dmsg2StringListMetadata {
            list_offset,
            list_length: walker.offset() as u32 - list_offset,
        })
    }
}

impl Dmsg2StringTable {
    fn parse_headers<T: ByteWalker>(walker: &mut T) -> Result<Dmsg2StringTableHeaders> {
        walker.expect_utf8_str("d_msg")?;
        walker.expect_utf8_str("\0\0\0")?;
        walker.expect(1u16)?;
        walker.expect(1u16)?;
        walker.expect(3u32)?;
        walker.expect(3u32)?;

        let file_bytes = walker.len() as u32;
        walker.expect(file_bytes)?;

        let header_bytes: u32 = walker.step()?;
        expect(HEADER_SIZE, header_bytes)?;

        let metadata_bytes: u32 = walker.step()?;

        walker.expect(0u32)?;

        let string_entry_bytes: u32 = walker.step()?;
        expect(
            file_bytes,
            HEADER_SIZE + metadata_bytes + string_entry_bytes,
        )?;

        let list_count: u32 = walker.step()?;

        walker.expect(1u32)?;
        walker.expect(0u64)?;
        walker.expect(0u64)?;

        Ok(Dmsg2StringTableHeaders {
            metadata_bytes,
            string_entry_bytes,
            list_count,
        })
    }

    fn parse_list_metadata<T: ByteWalker>(
        walker: &mut T,
        headers: &Dmsg2StringTableHeaders,
    ) -> Result<Vec<Dmsg2StringListMetadata>> {
        let mut list_metadata = Vec::with_capacity(headers.list_count as usize);

        for idx in 0..headers.list_count {
            let list_offset: i32 = !walker.step()?;
            let list_length: i32 = !walker.step()?;
            if list_length < 0
                || list_offset < 0
                || (list_offset + list_length) as u32 > headers.string_entry_bytes
            {
                return Err(anyhow!(
                    "Invalid length ({list_length}) and/or offset ({list_offset}) for block {idx}."
                ));
            }
            let list_offset = list_offset as u32;
            let list_length = list_length as u32;

            list_metadata.push(Dmsg2StringListMetadata {
                list_offset,
                list_length,
            })
        }

        Ok(list_metadata)
    }

    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Dmsg2StringTable> {
        let headers: Dmsg2StringTableHeaders = Self::parse_headers(walker)?;
        let list_metadata = Self::parse_list_metadata(walker, &headers)?;

        let start_of_entries = walker.offset() as u32;
        if start_of_entries - HEADER_SIZE != headers.metadata_bytes {
            return Err(anyhow!(
                "Metadata bytes didn't match: {} vs {}",
                start_of_entries - HEADER_SIZE,
                headers.metadata_bytes
            ));
        }

        let entries = list_metadata
            .into_iter()
            .enumerate()
            .map(|(idx, list_metadata)| {
                if walker.offset() as u32 - start_of_entries != list_metadata.list_offset {
                    return Err(anyhow!(
                        "Current offset for {idx} didn't match offset from list metadata."
                    ));
                }

                match Dmsg2StringList::parse(walker, list_metadata) {
                    Ok(block) => Ok((idx as u32, block)),
                    Err(err) => Err(anyhow!("Failed to parse string list {idx}: {err}")),
                }
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        if walker.remaining() != 0 {
            return Err(anyhow!(
                "Remaining bytes not parsed: {:02X?}",
                walker.read_bytes(walker.remaining())?
            ));
        }

        Ok(Dmsg2StringTable { lists: entries })
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        let entry_count = self.lists.len() as u32;
        let metadata_bytes = entry_count * LIST_METADATA_SIZE;
        let start_of_strings = HEADER_SIZE + metadata_bytes;

        // Skip headers and metadata to write strings first, such that the necessary lengths
        // can be gathered about the strings first.
        walker.goto(start_of_strings);

        let list_metadatas = self
            .lists
            .iter()
            .map(|(_, list)| list.write(walker))
            .collect::<Result<Vec<_>>>()?;

        let string_entry_bytes = walker.offset() as u32 - start_of_strings;

        // Go back and write in headers and metadata
        walker.goto(0);

        walker.write_str("d_msg");
        walker.skip(3);
        walker.write(1u16);
        walker.write(1u16);
        walker.write(3u32);
        walker.write(3u32);

        // File len
        walker.write(HEADER_SIZE + metadata_bytes + string_entry_bytes);

        // Header len
        walker.write(HEADER_SIZE);

        // Metadata len
        walker.write(metadata_bytes);
        walker.write(0u32);

        // String entries len
        walker.write(string_entry_bytes);

        // Amount of list entries
        walker.write(entry_count);

        walker.write(1u32);
        walker.write(0u64);
        walker.write(0u64);

        // Write list metadata
        for list_metadata in list_metadatas {
            walker.write::<u32>(!(list_metadata.list_offset - metadata_bytes - HEADER_SIZE));
            walker.write::<u32>(!list_metadata.list_length);
        }

        Ok(())
    }
}

impl DatFormat for Dmsg2StringTable {
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        Dmsg2StringTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        Dmsg2StringTable::parse_headers(walker)?;
        Ok(())
    }
}
