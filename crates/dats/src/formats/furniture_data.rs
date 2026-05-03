use anyhow::{Result, anyhow};
use common::{byte_walker::ByteWalker, writing_byte_walker::WritingByteWalker};
use serde_derive::{Deserialize, Serialize};

use crate::{dat_format::DatFormat, flags::FurniturePlacement, serde_hex};

const HEADER_SIZE: usize = 0x170;
const ENTRY_SIZE: usize = 8;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FurnitureEntry {
    placement: FurniturePlacement,
    model_no: u16,
    size_x: u8,
    size_z: u8,
    height: u16,
}

// Entry index is the general-items id: idx for idx < 512, else idx + 3072.
// Zero entries are kept; dropping any shifts every later id.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FurnitureData {
    #[serde(with = "serde_hex")]
    header: Vec<u8>,
    entries: Vec<FurnitureEntry>,
    #[serde(with = "serde_hex", default, skip_serializing_if = "Vec::is_empty")]
    trailing: Vec<u8>,
}

impl FurnitureData {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        if walker.len() < HEADER_SIZE || (walker.len() - HEADER_SIZE) % ENTRY_SIZE != 0 {
            return Err(anyhow!("Length does not match a furniture data DAT."));
        }

        let header = walker.take_bytes(HEADER_SIZE)?.to_vec();

        let entry_count = walker.remaining() / ENTRY_SIZE;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(FurnitureEntry {
                placement: FurniturePlacement::from_bits_retain(walker.step::<u16>()?),
                model_no: walker.step::<u16>()?,
                size_x: walker.step::<u8>()?,
                size_z: walker.step::<u8>()?,
                height: walker.step::<u16>()?,
            });
        }

        let trailing = walker.take_bytes(walker.remaining())?.to_vec();

        Ok(FurnitureData {
            header,
            entries,
            trailing,
        })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.write_bytes(&self.header);
        for entry in &self.entries {
            walker.write::<u16>(entry.placement.bits());
            walker.write(entry.model_no);
            walker.write(entry.size_x);
            walker.write(entry.size_z);
            walker.write(entry.height);
        }
        walker.write_bytes(&self.trailing);
        Ok(())
    }
}

impl DatFormat for FurnitureData {
    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        FurnitureData::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        if walker.len() < HEADER_SIZE || (walker.len() - HEADER_SIZE) % ENTRY_SIZE != 0 {
            return Err(anyhow!("Length does not match a furniture data DAT."));
        }
        walker.expect_utf8_str("myro")?;
        Ok(())
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{dat_format::DatFormat, flags::FurniturePlacement};

    use super::FurnitureData;

    #[test]
    pub fn furniture_data() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out_path = dat_path.join("resources/test/output/furniture_data.yml");
        dat_path.push("resources/test/furniture_data.DAT");

        FurnitureData::check_path(&dat_path).unwrap();
        let res = FurnitureData::from_path_checked_yaml(&dat_path).unwrap();

        assert_eq!(res.entries.len(), 1086);

        let entry = res.entries.get(10).unwrap();
        assert_eq!(entry.placement, FurniturePlacement::CanPutOn);
        assert_eq!(entry.model_no, 10);
        assert_eq!(entry.size_x, 1);
        assert_eq!(entry.size_z, 1);
        assert_eq!(entry.height, 10);

        fs::create_dir_all(out_path.parent().unwrap()).unwrap();
        let file = fs::File::create(out_path).unwrap();
        serde_yaml::to_writer(file, &res).unwrap();
    }
}
