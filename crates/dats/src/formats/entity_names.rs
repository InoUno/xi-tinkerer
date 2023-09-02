use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use common::{byte_walker::ByteWalker, writing_byte_walker::WritingByteWalker};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::dat_format::DatFormat;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityNames {
    pub names: Vec<EntityName>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityName {
    id: u32,
    name: String,
}

pub fn get_entity_names_zone(path: &PathBuf) -> Option<u16> {
    let mut file = File::open(path).ok()?;

    let mut four_bytes = [0u8; 4];
    file.read_exact(&mut four_bytes).ok()?;

    let starts_with_none = four_bytes == "none".as_bytes();
    if !starts_with_none {
        return None;
    }

    let mut first_id = 0;
    let mut current_id_pos = 0;
    while first_id == 0 {
        current_id_pos += 1;
        file.seek(SeekFrom::Start(0x1C + current_id_pos * 0x20))
            .ok()?;
        file.read_exact(&mut four_bytes).ok()?;

        first_id = u32::from_le_bytes(four_bytes);
    }

    Some(((first_id >> 12) & 0xFFF) as u16)
}

impl EntityNames {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<EntityNames> {
        walker.expect_utf8_str("none")?;

        Ok(EntityNames {
            names: EntityNames::read_names(walker)?,
        })
    }

    fn read_names<T: ByteWalker>(walker: &mut T) -> Result<Vec<EntityName>> {
        walker.goto(32);

        let mut names = vec![];
        while walker.remaining() >= 32 {
            names.push(parse_next_entity_name(walker)?);
        }

        Ok(names)
    }
}

fn parse_next_entity_name<T: ByteWalker>(walker: &mut T) -> Result<EntityName> {
    let name = Decoder::decode_simple(walker.take_bytes(28)?)?;
    let id: u32 = walker.step()?;

    Ok(EntityName { id, name })
}

impl DatFormat for EntityNames {
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.write_bytes("none".as_bytes());
        walker.goto(32);

        for name in self.names.iter() {
            let name_bytes = Encoder::encode_simple(&name.name)?;
            if name_bytes.len() > 28 {
                return Err(anyhow!(
                    "Name can at most be 28 bytes long: '{}'",
                    name.name
                ));
            }

            walker.write_bytes(&name_bytes);
            walker.skip(28 - name_bytes.len());
            walker.write(name.id);
        }

        Ok(())
    }

    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        EntityNames::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        walker.expect_utf8_str("none")?;
        Ok(())
    }
}
