use anyhow::{Result, anyhow};
use common::{byte_walker::ByteWalker, writing_byte_walker::WritingByteWalker};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

const BRACKET_REGION_SIZE: usize = 32;
const PLAINTEXT_REGION_SIZE: usize = 32;
const BRACKET_OPEN: [u8; 2] = [0x81, 0x79];
const BRACKET_CLOSE: [u8; 2] = [0x81, 0x7A];

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoTranslate {
    categories: Vec<AtCategory>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtCategory {
    id: u8,
    name: String,
    entries: Vec<AtEntry>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtEntry {
    id: u8,
    text: String,
}

impl AutoTranslate {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        let mut categories = Vec::new();
        while walker.remaining() > 0 {
            categories.push(AtCategory::parse(walker)?);
        }
        Ok(AutoTranslate { categories })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        for cat in &self.categories {
            cat.write(walker)?;
        }
        Ok(())
    }
}

impl AtCategory {
    fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        walker.expect_n_msg::<u8>(0x02, 2, "Auto-translate category lead-in")?;
        let id = walker.step::<u8>()?;
        walker.expect_msg::<u8>(0, "Auto-translate category lead-in reserved")?;

        let bracket_region = walker.take_bytes(BRACKET_REGION_SIZE)?.to_vec();
        if &bracket_region[0..2] != BRACKET_OPEN {
            return Err(anyhow!("Missing auto-translate bracket open marker."));
        }
        let close_pos = bracket_region
            .windows(2)
            .skip(2)
            .position(|w| w == BRACKET_CLOSE)
            .map(|p| p + 2)
            .ok_or_else(|| anyhow!("Missing auto-translate bracket close marker."))?;
        let name_bytes = &bracket_region[2..close_pos];
        if bracket_region[close_pos + 2..].iter().any(|&b| b != 0) {
            return Err(anyhow!("Auto-translate bracket region padding not zero."));
        }

        let plaintext_region = walker.take_bytes(PLAINTEXT_REGION_SIZE)?;
        if &plaintext_region[..name_bytes.len()] != name_bytes {
            return Err(anyhow!(
                "Auto-translate plaintext name does not match bracketed name."
            ));
        }
        if plaintext_region[name_bytes.len()..].iter().any(|&b| b != 0) {
            return Err(anyhow!("Auto-translate plaintext region padding not zero."));
        }

        let name = Decoder::decode_simple(name_bytes)?;

        let entry_count = walker.step::<u32>()?;
        let byte_size = walker.step::<u32>()? as usize;
        let entries_end = walker.offset() + byte_size;

        let mut entries = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            walker.expect_n_msg::<u8>(0x02, 2, "Auto-translate entry marker")?;
            let cat_id = walker.step::<u8>()?;
            if cat_id != id {
                return Err(anyhow!(
                    "Auto-translate entry cat_id mismatch: {} vs {}.",
                    cat_id,
                    id
                ));
            }
            let entry_id = walker.step::<u8>()?;
            let str_len = walker.step::<u8>()? as usize;
            let text = Decoder::decode_simple(walker.take_bytes(str_len)?)?;
            entries.push(AtEntry {
                id: entry_id,
                text,
            });
        }

        if walker.offset() != entries_end {
            return Err(anyhow!("Auto-translate entries section byte_size mismatch."));
        }

        Ok(AtCategory { id, name, entries })
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.write::<u8>(0x02);
        walker.write::<u8>(0x02);
        walker.write::<u8>(self.id);
        walker.write::<u8>(0x00);

        let name_bytes = Encoder::encode_simple(&self.name)?;
        if name_bytes.len() + 4 > BRACKET_REGION_SIZE {
            return Err(anyhow!(
                "Auto-translate category name too long: '{}'",
                self.name
            ));
        }

        let mut bracket = vec![0u8; BRACKET_REGION_SIZE];
        bracket[0..2].copy_from_slice(&BRACKET_OPEN);
        bracket[2..2 + name_bytes.len()].copy_from_slice(&name_bytes);
        let close_offset = 2 + name_bytes.len();
        bracket[close_offset..close_offset + 2].copy_from_slice(&BRACKET_CLOSE);
        walker.write_bytes(&bracket);

        let mut plaintext = vec![0u8; PLAINTEXT_REGION_SIZE];
        plaintext[..name_bytes.len()].copy_from_slice(&name_bytes);
        walker.write_bytes(&plaintext);

        let mut entries_buf: Vec<u8> = Vec::new();
        for entry in &self.entries {
            let mut text_bytes = Encoder::encode_simple(&entry.text)?;
            text_bytes.push(0);
            if text_bytes.len() > 0xFF {
                return Err(anyhow!(
                    "Auto-translate entry text too long: '{}'",
                    entry.text
                ));
            }
            entries_buf.push(0x02);
            entries_buf.push(0x02);
            entries_buf.push(self.id);
            entries_buf.push(entry.id);
            entries_buf.push(text_bytes.len() as u8);
            entries_buf.extend_from_slice(&text_bytes);
        }

        walker.write::<u32>(self.entries.len() as u32);
        walker.write::<u32>(entries_buf.len() as u32);
        walker.write_bytes(&entries_buf);

        Ok(())
    }
}

impl DatFormat for AutoTranslate {
    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        AutoTranslate::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        let leadin = walker.take_bytes(4)?;
        if leadin[0] != 0x02 || leadin[1] != 0x02 || leadin[3] != 0 {
            return Err(anyhow!("Does not match an auto-translate DAT."));
        }
        if walker.take_bytes(2)? != BRACKET_OPEN {
            return Err(anyhow!("Does not match an auto-translate DAT."));
        }
        Ok(())
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::dat_format::DatFormat;

    use super::AutoTranslate;

    #[test]
    pub fn auto_translate() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out_path = dat_path.join("resources/test/output/auto_translate.yml");
        dat_path.push("resources/test/auto_translate.DAT");

        AutoTranslate::check_path(&dat_path).unwrap();
        let res = AutoTranslate::from_path_checked_yaml(&dat_path).unwrap();

        assert_eq!(res.categories.len(), 42);

        let greetings = res.categories.first().unwrap();
        assert_eq!(greetings.id, 1);
        assert_eq!(greetings.name, "Greetings");
        assert_eq!(greetings.entries.len(), 20);
        assert_eq!(greetings.entries[0].id, 1);
        assert_eq!(greetings.entries[0].text, "Nice to meet you.");

        fs::create_dir_all(out_path.parent().unwrap()).unwrap();
        let file = fs::File::create(out_path).unwrap();
        serde_yaml::to_writer(file, &res).unwrap();
    }
}
