use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use common::{byte_walker::ByteWalker, expect_msg, writing_byte_walker::WritingByteWalker};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct XiStringTable {
    strings: BTreeMap<u32, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct XiStringMeta {
    offset: u32,
    size: u16,
}

const HEADER_SIZE: u32 = 0x38;

impl XiStringTable {
    fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        walker.expect_utf8_str("XISTRING\0\0")?;
        walker.expect::<u16>(2)?;

        walker.expect_n_msg::<u8>(0, 20, "Initial padding")?;

        let file_bytes = walker.len() as u32;
        walker.expect_msg(file_bytes, "File size")?;

        let entry_count: u32 = walker.step()?;
        let meta_bytes: u32 = walker.step()?;
        let data_bytes: u32 = walker.step()?;

        if meta_bytes != entry_count * 12 || file_bytes != HEADER_SIZE + meta_bytes + data_bytes {
            return Err(anyhow!("Invalid header values."));
        }

        let unknown1: u32 = walker.step()?;
        if unknown1 != 0 {
            return Err(anyhow!("unknown1 is {}", unknown1));
        }

        let unknown2: u32 = walker.step()?;
        // TODO: this value is different in at least IngameMessages2, where it is 304231515
        if unknown2 != 304091210 {
            return Err(anyhow!("unknown2 is {}", unknown2));
        }

        // Read metadata
        let mut metas = vec![];
        for _ in 0..entry_count {
            let offset: u32 = walker.step()?;
            let size: u16 = walker.step()?;

            // TODO: TimeAndPronouns has a 1 here instead of 0
            walker.expect_msg(0u16, "Unknown meta 1")?;

            walker.expect_msg(0u16, "Unknown meta 2")?;
            walker.expect_msg(0u16, "Unknown meta 3")?;

            metas.push(XiStringMeta { offset, size });
        }

        // Read the strings
        let mut strings = BTreeMap::default();
        for (idx, meta) in metas.into_iter().enumerate() {
            expect_msg(
                HEADER_SIZE + meta_bytes + meta.offset,
                walker.offset() as u32,
                "Offset of string",
            )?;

            let string_bytes = walker.take_bytes(meta.size as usize)?;
            let string = Decoder::decode_simple(string_bytes)?;
            strings.insert(idx as u32, string);
        }

        Ok(XiStringTable { strings })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.write_str("XISTRING\0\0");
        walker.write::<u16>(2);

        for _ in 0..20 {
            walker.write::<u8>(0);
        }

        let encoded_strings = self
            .strings
            .iter()
            .map(|(idx, string)| {
                let encoded = Encoder::encode_simple(&string)?;

                Ok((idx, encoded))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;

        let entry_count = self.strings.keys().max().copied().unwrap_or_default() as u32 + 1;
        let data_bytes = encoded_strings
            .iter()
            .map(|(_, str)| str.len())
            .sum::<usize>() as u32
            + entry_count; // A zero-byte to end each entry

        let meta_bytes = entry_count * 12;
        let file_bytes = HEADER_SIZE + meta_bytes + data_bytes;

        walker.write(file_bytes);

        walker.write(entry_count);
        walker.write(meta_bytes);
        walker.write(data_bytes);

        walker.write::<u32>(0); // unknown1
        walker.write::<u32>(304091210); // unknown2

        // Write metadata for strings
        let mut current_string_offset = 0;
        for idx in 0..entry_count {
            let string_len = encoded_strings
                .get(&idx)
                .map(|str| str.len())
                .unwrap_or_default()
                + 1; // 1 extra byte for string end

            walker.write(current_string_offset);
            walker.write(string_len as u16);

            // Unknowns
            walker.write(0u16);
            walker.write(0u16);
            walker.write(0u16);

            current_string_offset += string_len as u32;
        }

        // Write the strings
        for idx in 0..entry_count {
            if let Some(encoded_string) = encoded_strings.get(&idx) {
                walker.write_bytes(&encoded_string);
            }
            walker.write::<u8>(0); // End of string
        }

        Ok(())
    }
}

impl DatFormat for XiStringTable {
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        XiStringTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        walker.expect_utf8_str("XISTRING\0\0")?;
        walker.expect::<u16>(2)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{dat_format::DatFormat, formats::xistring_table::XiStringTable};

    #[test]
    pub fn pol_messages() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/pol_messages.DAT");

        XiStringTable::check_path(&dat_path).unwrap();
        let res = XiStringTable::from_path_checked(&dat_path).unwrap();

        assert_eq!(
            res.strings.get(&0).unwrap(),
            &"Searching for lobby server.".to_string()
        );

        assert_eq!(
            res.strings.get(&104).unwrap(),
            &"Select a character to play.".to_string()
        );
    }
}
