use anyhow::{anyhow, Result};
use common::{byte_walker::ByteWalker, expect, writing_byte_walker::WritingByteWalker};
use encoding::decoder::Decoder;

use crate::dat_format::DatFormat;

pub struct DmsgStringTable1 {
    entries: Vec<DmsgStringTable1Entry>,
}

pub struct DmsgStringTable1Entry {
    unknown1: u32,
    unknown2: u16,
    unknown3: u32,
    unknown4: u32,
    unknown5: u32,
    unknown6: u32,
    unknown7: u32,
    unknown8: u16,
    unknown9: u16,
    string: String,
}

const HEADER_SIZE: u32 = 0x38;
const ENTRY_SIZE: u32 = 0x24;

impl DmsgStringTable1Entry {
    pub fn write<T: WritingByteWalker>(
        &self,
        walker: &mut T,
        current_data_offset: &mut u32,
        idx: u32,
    ) {
        walker.goto(HEADER_SIZE + ENTRY_SIZE * idx);

        walker.write(current_data_offset.clone());

        walker.write(self.unknown1);

        let string_len = self.string.len() as u32;
        walker.write(string_len);

        walker.write(self.unknown2);
        walker.write(self.unknown3);
        walker.write(self.unknown4);
        walker.write(self.unknown5);
        walker.write(self.unknown6);
        walker.write(self.unknown7);
        walker.write(self.unknown8);
        walker.write(self.unknown9);

        walker.goto(*current_data_offset);

        walker.write_str(&self.string);

        *current_data_offset += string_len;
    }
}

fn parse_dmsg_string_table_entry<T: ByteWalker>(
    walker: &mut T,
    data_start: u32,
    data_bytes: u32,
    idx: u32,
) -> Result<DmsgStringTable1Entry> {
    walker.goto(HEADER_SIZE + ENTRY_SIZE * idx);

    let data_offset: u32 = walker.step()?;

    let unknown1: u32 = walker.step()?;

    let string_len: i16 = walker.step()?;

    let unknown2: u16 = walker.step()?;
    let unknown3: u32 = walker.step()?;
    let unknown4: u32 = walker.step()?;
    let unknown5: u32 = walker.step()?;
    let unknown6: u32 = walker.step()?;
    let unknown7: u32 = walker.step()?;
    let unknown8: u16 = walker.step()?;
    let unknown9: u16 = walker.step()?;

    if string_len < 0 || (data_offset + string_len as u32) > data_bytes {
        return Err(anyhow!(
            "Invalid offset ({data_offset}) or string length ({string_len}) to fit into {data_bytes} bytes.",
        ));
    }

    walker.goto(data_start + data_offset);
    Ok(DmsgStringTable1Entry {
        unknown1,
        unknown2,
        unknown3,
        unknown4,
        unknown5,
        unknown6,
        unknown7,
        unknown8,
        unknown9,
        string: Decoder::decode_simple(walker.take_bytes(string_len as usize)?)?,
    })
}

fn parse_dmsg_string_table<T: ByteWalker>(walker: &mut T) -> Result<DmsgStringTable1> {
    walker.expect_utf8_str("d_msg")?;
    walker.expect_utf8_str("\0\0\0")?;
    walker.expect(1u16)?;
    walker.expect(0u32)?;
    walker.expect(2u16)?;
    walker.expect(3u32)?;

    let entry_count: u32 = walker.step()?;
    walker.expect(1u32)?;

    let bytes_len = walker.len() as u32;
    walker.expect(bytes_len)?;

    let header_bytes: u32 = walker.step()?;
    expect(HEADER_SIZE, header_bytes)?;

    let entry_bytes: u32 = walker.step()?;
    expect(36 * entry_count, entry_bytes)?;

    let data_bytes: u32 = walker.step()?;
    expect(bytes_len, HEADER_SIZE + entry_bytes + data_bytes)?;

    walker.expect(0u32)?;
    walker.expect(0u32)?;
    walker.expect(0u32)?;

    let data_start = HEADER_SIZE + entry_bytes;

    let mut data = vec![];
    for idx in 0..entry_count {
        match parse_dmsg_string_table_entry(walker, data_start, data_bytes, idx) {
            Ok(block) => data.push(block),
            Err(err) => {
                return Err(anyhow!("Failed to parse block: {err}"));
            }
        }
    }

    Ok(DmsgStringTable1 { entries: data })
}

impl DatFormat for DmsgStringTable1 {
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.write_str("d_msg");
        walker.skip(3);
        walker.write(1u16);
        walker.write(0u32);
        walker.write(2u16);
        walker.write(3u32);

        walker.write(self.entries.len() as u32);
        walker.write(1u32);

        let all_bytes_len_offset = walker.offset() as u32;
        walker.skip(4); // Reserved for full bytes len

        walker.write(HEADER_SIZE); // Header len

        let entry_bytes_len = ENTRY_SIZE * self.entries.len() as u32;
        walker.write(entry_bytes_len);

        let data_bytes_len_offset = walker.offset() as u32;
        walker.skip(4); // Reserved for data bytes len

        walker.skip(4 * 3);

        // Write all entries
        let data_start = HEADER_SIZE + entry_bytes_len;
        let mut current_data_offset = data_start;
        for (idx, entry) in self.entries.iter().enumerate() {
            entry.write(walker, &mut current_data_offset, idx as u32);
        }

        // Go back and write in length of data and full DAT
        let data_bytes_len = current_data_offset - data_start;
        walker.goto(data_bytes_len_offset);
        walker.write(data_bytes_len);

        let bytes_len = HEADER_SIZE + entry_bytes_len + data_bytes_len;
        walker.goto(all_bytes_len_offset);
        walker.write(bytes_len);

        Ok(())
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        parse_dmsg_string_table(walker)
    }

    fn check_type<T: ByteWalker>(_walker: &mut T) -> Result<()> {
        Err(anyhow!("Not yet implemented"))
    }
}
