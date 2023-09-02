use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use common::{byte_walker::ByteWalker, writing_byte_walker::WritingByteWalker};
use encoding::decoder::Decoder;
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StringTableEntry {
    id: u32,
    string: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StringTable {
    entries: BTreeMap<u32, StringTableEntry>,
}

impl StringTable {
    fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        if walker.len() % 0x40 != 0 {
            return Err(anyhow!("Not a string table."));
        }

        let entry_count = walker.len() / 0x40;

        let mut entries = BTreeMap::new();
        for idx in 0..entry_count {
            let id: u32 = walker.step()?;
            let string = Decoder::decode_simple(walker.take_bytes(59)?)?;
            entries.insert(idx as u32, StringTableEntry { id, string });

            walker.expect_msg::<u8>(0xFF, "String entry expected to be ended by 0xFF.")?;
        }

        Ok(StringTable { entries })
    }
}

impl DatFormat for StringTable {
    fn write<T: WritingByteWalker>(&self, _walker: &mut T) -> Result<()> {
        Err(anyhow!("Not yet implemented"))
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        StringTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        if walker.len() % 0x40 != 0 {
            return Err(anyhow!("Does not have a size that matches a string table."));
        }

        let string_end_byte = walker.read_at::<u8>(0x40)?;
        if string_end_byte != 0xFF {
            return Err(anyhow!("Expected strings to be ended by 0xFF."));
        }
        Ok(())
    }
}
