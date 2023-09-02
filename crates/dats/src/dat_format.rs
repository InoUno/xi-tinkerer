use anyhow::{anyhow, Result};
use common::{
    byte_walker::{BufferedByteWalker, ByteWalker},
    checking_byte_walker::CheckingByteWalker,
    file_byte_walker::FileByteWalker,
    vec_byte_walker::VecByteWalker,
    writing_byte_walker::WritingByteWalker,
};
use std::{
    cmp::min,
    fs::{self},
    path::PathBuf,
};

pub trait DatFormat: Sized {
    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self>;
    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()>;
    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()>;
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut walker = VecByteWalker::new();
        self.write(&mut walker)?;
        Ok(walker.into_vec())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut walker = BufferedByteWalker::on(bytes);
        Self::from(&mut walker)
    }

    fn from_path(path: &PathBuf) -> Result<Self> {
        let mut walker = FileByteWalker::from_path(path)?;
        Self::from(&mut walker)
    }

    fn check_path(path: &PathBuf) -> Result<()> {
        let mut walker = FileByteWalker::from_path(path)?;
        Self::check_type(&mut walker)
    }

    fn from_bytes_checked(bytes: &[u8]) -> Result<Self> {
        let res = Self::from_bytes(bytes)?;
        if res.to_bytes()? != bytes {
            return Err(anyhow!("Failed round-trip test."));
        }
        Ok(res)
    }

    fn from_path_checked_during(path: &PathBuf) -> Result<Self> {
        let original_walker = FileByteWalker::from_path(path)?;
        let mut checking_walker = CheckingByteWalker::new(original_walker);

        let res = Self::from_path(path)?;
        res.write(&mut checking_walker)?;

        if checking_walker.remaining() != 0 {
            return Err(anyhow!(
                "Missing {} bytes written.",
                checking_walker.remaining()
            ));
        }

        Ok(res)
    }

    fn from_path_checked(path: &PathBuf) -> Result<Self> {
        let original_bytes = fs::read(path)?;
        let res = Self::from_path(path)?;
        let re_encoded_bytes = res.to_bytes()?;
        if re_encoded_bytes.len() != original_bytes.len() || re_encoded_bytes != original_bytes {
            let first_diff_idx = original_bytes
                .iter()
                .zip(re_encoded_bytes.iter())
                .enumerate()
                .find_map(|(idx, (original_byte, encoded_byte))| {
                    if original_byte != encoded_byte {
                        Some(idx)
                    } else {
                        None
                    }
                });

            if first_diff_idx.is_none() {
                return Err(anyhow!(
                    "Round-trip test failed because of non-matching lengths: {} vs {}",
                    original_bytes.len(),
                    re_encoded_bytes.len()
                ));
            }
            let first_diff_idx = first_diff_idx.unwrap();

            let context_start = first_diff_idx.saturating_sub(10);
            let context_end = min(
                re_encoded_bytes.len(),
                min(original_bytes.len(), first_diff_idx + 10),
            );

            return Err(anyhow!(
                "Failed round-trip test for '{}' at index {}:\n{:02X?}\n{:02X?}",
                path.display(),
                first_diff_idx,
                &original_bytes[context_start..context_end],
                &re_encoded_bytes[context_start..context_end],
            ));
        }
        Ok(res)
    }
}
