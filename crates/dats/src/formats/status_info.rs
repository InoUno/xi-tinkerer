use anyhow::{anyhow, Ok, Result};
use common::{
    byte_walker::{BufferedByteWalker, ByteWalker},
    vec_byte_walker::VecByteWalker,
    writing_byte_walker::WritingByteWalker,
};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::serde_base64;
use crate::{
    dat_format::DatFormat,
    utils::{decode_data_block, encode_data_block},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusInfo {
    id: u16,
    description: String,

    flag: u16,

    #[serde(with = "serde_base64")]
    icon_bytes: Vec<u8>,
}

impl StatusInfo {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<StatusInfo> {
        let mut data_bytes = walker.take_bytes(0x280)?.to_vec();
        decode_data_block(&mut data_bytes);

        let mut data_walker = BufferedByteWalker::on(data_bytes);

        let id = data_walker.step::<u16>()?;

        let flag = data_walker.step::<u16>()?;

        data_walker.expect::<u32>(1)?;
        data_walker.expect::<u32>(12)?;
        data_walker.expect::<u32>(0)?;
        data_walker.expect::<u32>(1)?;

        data_walker.expect_n_msg::<u8>(0, 24, "Padding after unknowns")?;

        let description = Decoder::decode_simple(data_walker.step_until(0)?)?;

        let icon_size = walker.step::<u32>()?;
        let icon_bytes = walker.take_bytes(icon_size as usize)?.to_vec();

        let icon_padding = walker.remaining() % ENTRY_SIZE - 1;
        walker.expect_n_msg::<u8>(0, icon_padding, "Padding after icon")?;
        walker.expect_msg::<u8>(0xFF, "End of status info byte")?;

        Ok(StatusInfo {
            id,
            flag,
            description,
            icon_bytes,
        })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        let mut data_walker = VecByteWalker::with_size(0x280);

        data_walker.write(self.id);
        data_walker.write(self.flag);

        data_walker.write::<u32>(1);
        data_walker.write::<u32>(12);
        data_walker.write::<u32>(0);
        data_walker.write::<u32>(1);

        data_walker.skip(24);

        let description = Encoder::encode_simple(&self.description)?;
        data_walker.write_bytes(&description);

        let mut data_bytes = data_walker.into_vec();
        encode_data_block(&mut data_bytes);

        walker.write_bytes(&data_bytes);

        walker.write(self.icon_bytes.len() as u32);
        walker.write_bytes(&self.icon_bytes);

        let icon_padding = ENTRY_SIZE - walker.offset() % ENTRY_SIZE - 1;
        walker.skip(icon_padding);

        walker.write::<u8>(0xFF);

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatusInfoTable {
    status_infos: Vec<StatusInfo>,
}

const ENTRY_SIZE: usize = 0x1800;

impl StatusInfoTable {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        if walker.len() % ENTRY_SIZE != 0 {
            return Err(anyhow!("Length does not match a status info DAT."));
        }

        let entry_count = walker.len() / ENTRY_SIZE;
        let mut status_infos = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            status_infos.push(StatusInfo::parse(walker)?);
        }

        Ok(StatusInfoTable { status_infos })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.set_size(self.status_infos.len() * ENTRY_SIZE);

        for status_info in &self.status_infos {
            status_info.write(walker)?;
        }

        Ok(())
    }
}

impl DatFormat for StatusInfoTable {
    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        StatusInfoTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        if walker.len() % ENTRY_SIZE != 0 {
            return Err(anyhow!("Length does not match a status info DAT."));
        }

        // Parse one status info to check.
        StatusInfo::parse(walker)?;

        Ok(())
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::dat_format::DatFormat;

    use super::StatusInfoTable;

    #[test]
    pub fn status_infos() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/status_infos.DAT");

        StatusInfoTable::check_path(&dat_path).unwrap();
        let res = StatusInfoTable::from_path_checked_during(&dat_path).unwrap();

        assert_eq!(
            res.status_infos[0].description,
            "You have been knocked unconscious.".to_string()
        );

        assert_eq!(
            res.status_infos[614].description,
            "Ullegore is making you forget the true meaning of \"fun\"!".to_string()
        );
    }
}
