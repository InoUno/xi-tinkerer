use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use common::{byte_walker::ByteWalker, get_padding, writing_byte_walker::WritingByteWalker};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dialog {
    pub entries: BTreeMap<u32, String>,
}

const DIALOG_MASK: u32 = 0x80808080;
const DIALOG_U8_MASK: u8 = 0x80;

impl Dialog {
    fn parse_dialog_string<T: ByteWalker>(walker: &mut T, end: u32) -> Result<String> {
        let bytes = walker
            .take_bytes(end as usize - walker.offset())?
            .into_iter()
            .map(|byte| byte ^ DIALOG_U8_MASK)
            .collect::<Vec<_>>();

        let string = Decoder::decode_dialog(&bytes)?;

        Ok(string)
    }

    fn get_header_values<T: ByteWalker>(walker: &mut T) -> Result<(u32, u32)> {
        let size_info = walker.step::<u32>()?;

        if size_info == 0 {
            return Err(anyhow!("Possible empty dialog DAT."));
        }

        let file_size = (size_info ^ 0x10000000) + 4;

        if file_size != walker.len() as u32 {
            return Err(anyhow!(
                "Invalid file size {} with byte count {}.",
                file_size,
                walker.len()
            ));
        }

        let shifted_string_count = walker.step::<u32>()? ^ DIALOG_MASK;
        if shifted_string_count % 4 != 0
            || shifted_string_count > walker.len() as u32
            || shifted_string_count < 8
        {
            return Err(anyhow!(
                "Invalid shifted string count {} with byte count {}.",
                shifted_string_count,
                walker.len()
            ));
        }

        Ok((file_size, shifted_string_count >> 2))
    }

    fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        let (file_size, string_count) = Self::get_header_values(walker)?;

        let mut string_ends = (0..string_count - 1)
            .into_iter()
            .map(|_| walker.step::<u32>().map(|end| (end ^ DIALOG_MASK) + 4))
            .collect::<Result<Vec<_>>>()?;

        string_ends.push(file_size);

        let result = Dialog {
            entries: string_ends
                .into_iter()
                .enumerate()
                .map(|(idx, end)| Ok((idx as u32, Self::parse_dialog_string(walker, end)?)))
                .collect::<Result<_>>()?,
        };

        Ok(result)
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        let encoded_strings = self
            .entries
            .iter()
            .map(|(_, string)| Encoder::encode_dialog(string))
            .collect::<Result<Vec<_>>>()?;

        // Calculate size of the DAT
        let string_lengths_header_end = 4 + encoded_strings.len() * 4;
        let mut file_size: usize = string_lengths_header_end
            + encoded_strings
                .iter()
                .map(|bytes| bytes.len())
                .sum::<usize>();

        // Add padding
        file_size += get_padding(file_size);

        walker.set_size(file_size);

        // Write header with file size and string endings
        walker.write((file_size as u32 ^ 0x10000000) - 4);
        walker.write(((encoded_strings.len() as u32) << 2) ^ DIALOG_MASK);

        // Write the ending index for each string except the last one.
        let mut encoded_strings_iter = encoded_strings.iter();
        let mut current_ending =
            string_lengths_header_end + encoded_strings_iter.next().unwrap().len() - 4;

        for encoded_string in encoded_strings_iter {
            walker.write((current_ending as u32) ^ DIALOG_MASK);
            current_ending += encoded_string.len();
        }

        // Write the strings
        for mut encoded_string in encoded_strings {
            encoded_string.iter_mut().for_each(|b| {
                *b ^= DIALOG_U8_MASK;
            });
            walker.write_bytes(&encoded_string);
        }

        for _ in 0..walker.remaining() {
            walker.write(DIALOG_U8_MASK);
        }

        Ok(())
    }
}

impl DatFormat for Dialog {
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        Dialog::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        Dialog::get_header_values(walker)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{dat_format::DatFormat, formats::dialog::Dialog};

    #[test]
    pub fn whitegate() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/dialog_whitegate.DAT");

        Dialog::check_path(&dat_path).unwrap();
        let res = Dialog::from_path_checked(&dat_path).unwrap();

        assert_eq!(res.entries.get(&129).unwrap(), "You observe no changes.");
    }
}
