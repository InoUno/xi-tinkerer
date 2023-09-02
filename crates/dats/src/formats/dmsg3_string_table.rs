use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use common::{byte_walker::ByteWalker, expect, writing_byte_walker::WritingByteWalker};
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

use super::dmsg::DmsgStringList;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Dmsg3StringTable {
    pub bytes_per_entry: u32,
    pub flip_bytes: bool,
    pub lists: BTreeMap<u32, DmsgStringList>,
}

#[derive(Debug)]
struct Dmsg3StringTableHeaders {
    bytes_per_entry: u32,
    flip_bytes: bool,
    entry_count: u32,
}

const HEADER_SIZE: u32 = 0x40;

impl Dmsg3StringTable {
    fn parse_headers<T: ByteWalker>(walker: &mut T) -> Result<Dmsg3StringTableHeaders> {
        walker.expect_utf8_str("d_msg")?;
        walker.expect_utf8_str("\0\0\0")?;

        let flag1 = walker.step::<u16>()?;
        if flag1 != 1 {
            return Err(anyhow!("Unexpected first flag: {flag1}"));
        }

        let flag2 = walker.step::<u16>()?;
        if flag2 != 0 && flag2 != 1 {
            return Err(anyhow!("Unexpected second flag: {flag2}"));
        }
        let flip_bytes = flag2 == 1;

        walker.expect(3u32)?;
        walker.expect(3u32)?;

        let file_bytes = walker.len() as u32;
        walker.expect(file_bytes)?;

        let header_bytes: u32 = walker.step()?;
        expect(HEADER_SIZE, header_bytes)?;

        walker.expect(0u32)?;

        let bytes_per_entry: u32 = walker.step()?;

        let data_bytes: u32 = walker.step()?;

        let entry_count: u32 = walker.step()?;
        expect(file_bytes, HEADER_SIZE + data_bytes)?;
        expect(data_bytes, entry_count * bytes_per_entry)?;

        walker.expect(1u32)?;
        walker.expect(0u64)?;
        walker.expect(0u64)?;

        Ok(Dmsg3StringTableHeaders {
            bytes_per_entry,
            flip_bytes,
            entry_count,
        })
    }

    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Dmsg3StringTable> {
        let headers: Dmsg3StringTableHeaders = Self::parse_headers(walker)?;

        let mut lists = BTreeMap::default();
        for idx in 0..headers.entry_count {
            lists.insert(
                idx,
                DmsgStringList::parse(walker, headers.flip_bytes, headers.bytes_per_entry)?,
            );
        }

        Ok(Dmsg3StringTable {
            bytes_per_entry: headers.bytes_per_entry,
            flip_bytes: headers.flip_bytes,
            lists,
        })
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        let entry_count = self.lists.len() as u32;
        let data_size = entry_count * self.bytes_per_entry;

        // Go back and write in headers and metadata
        walker.goto(0);

        walker.write_str("d_msg");
        walker.skip(3);

        walker.write(1u16);
        walker.write(if self.flip_bytes { 1u16 } else { 0u16 });

        walker.write(3u32);
        walker.write(3u32);

        // File len
        walker.write::<u32>(HEADER_SIZE + data_size);

        // Header len
        walker.write::<u32>(HEADER_SIZE);

        walker.write::<u32>(0x00);

        // Bytes per entry
        walker.write::<u32>(self.bytes_per_entry);

        // Data entries
        walker.write::<u32>(data_size);

        // Amount of list entries
        walker.write::<u32>(entry_count);

        walker.write(1u32);
        walker.write(0u64);
        walker.write(0u64);

        let mut next_end = walker.offset() + self.bytes_per_entry as usize;
        for idx in 0..entry_count {
            let Some(list) = self.lists.get(&idx) else {
                walker.skip(self.bytes_per_entry as usize);
                continue;
            };

            list.write(walker, self.flip_bytes)?;

            if next_end < walker.offset() {
                let diff = self.bytes_per_entry as usize + walker.offset() - next_end;
                return Err(anyhow!(
                    "Entry {}, can't fit with given bytes per entry ({} vs {}):\n{:#?}",
                    idx,
                    self.bytes_per_entry,
                    diff,
                    list
                ));
            }
            let diff = next_end.saturating_sub(walker.offset());
            walker.write_bytes(&vec![if self.flip_bytes { u8::MAX } else { 0u8 }; diff]);
            next_end += self.bytes_per_entry as usize;
        }

        Ok(())
    }
}

impl DatFormat for Dmsg3StringTable {
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        Dmsg3StringTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        Dmsg3StringTable::parse_headers(walker)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{dat_format::DatFormat, formats::dmsg::DmsgContent};

    use super::Dmsg3StringTable;

    #[test]
    pub fn ability_names() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/ability_names.DAT");

        Dmsg3StringTable::check_path(&dat_path).unwrap();
        let res = Dmsg3StringTable::from_path_checked(&dat_path).unwrap();

        assert_eq!(
            res.lists.get(&1).unwrap().content[0],
            DmsgContent::String {
                string: "Combo".to_string()
            }
        );

        assert_eq!(
            res.lists.get(&2).unwrap().content[0],
            DmsgContent::String {
                string: "Shoulder Tackle".to_string()
            }
        );
    }

    #[test]
    pub fn key_items() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/key_items.DAT");

        Dmsg3StringTable::check_path(&dat_path).unwrap();
        let res = Dmsg3StringTable::from_path_checked_during(&dat_path).unwrap();

        assert_eq!(
            res.lists.get(&1).unwrap().content[0],
            DmsgContent::Number { number: 1 }
        );
        assert_eq!(
            res.lists.get(&1).unwrap().content[4],
            DmsgContent::String {
                string: "Zeruhn report".to_string()
            }
        );

        assert_eq!(
            res.lists.get(&1534).unwrap().content[0],
            DmsgContent::Number { number: 619 }
        );
        assert_eq!(
            res.lists.get(&1534).unwrap().content[4],
            DmsgContent::String {
                string: "All-You-Can-Ride Pass".to_string()
            }
        );
        assert_eq!(
            res.lists.get(&1534).unwrap().content[6],
            DmsgContent::String {
                string: "\nFor GM use only!".to_string()
            }
        );
    }
}
