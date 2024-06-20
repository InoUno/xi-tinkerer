use std::{collections::BTreeMap, fmt::Display, marker::PhantomData, ops::Deref, path::PathBuf};

use crate::{context::DatContext, dat_format::DatFormat};
use anyhow::{anyhow, Result};

pub type ZoneId = u16;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatId(u32);

impl From<u32> for DatId {
    fn from(value: u32) -> Self {
        DatId(value)
    }
}

impl DatId {
    pub fn get_ffxi_dat_path(&self, dat_context: &DatContext) -> Result<PathBuf, DatError> {
        let dat_relative_path = self.get_relative_dat_path(dat_context)?;

        Ok(dat_context.ffxi_path.join(dat_relative_path))
    }

    pub fn get_relative_dat_path(&self, dat_context: &DatContext) -> Result<PathBuf, DatError> {
        let dat_path = dat_context
            .id_map
            .get(&self)
            .ok_or(DatError::DatMappingNotFound(self.clone()))?;

        Ok(dat_path.to_path())
    }

    pub fn get_inner(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dat<T: DatFormat> {
    raw_id: DatId,
    _pd: PhantomData<fn() -> T>,
}

impl<T: DatFormat> Clone for Dat<T> {
    fn clone(&self) -> Self {
        Self {
            raw_id: self.raw_id.clone(),
            _pd: PhantomData,
        }
    }
}

impl<T: DatFormat> From<DatId> for Dat<T> {
    fn from(value: DatId) -> Self {
        Dat {
            raw_id: value,
            _pd: PhantomData,
        }
    }
}

impl<T: DatFormat> From<u32> for Dat<T> {
    fn from(value: u32) -> Self {
        Dat {
            raw_id: DatId(value),
            _pd: PhantomData,
        }
    }
}

impl<T: DatFormat> From<&Dat<T>> for DatId {
    fn from(value: &Dat<T>) -> Self {
        value.raw_id
    }
}

impl<T: DatFormat> From<Dat<T>> for DatId {
    fn from(value: Dat<T>) -> Self {
        value.raw_id
    }
}

impl<T: DatFormat> Display for Dat<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_id.0)
    }
}

impl<T: DatFormat> Deref for Dat<T> {
    type Target = DatId;

    fn deref(&self) -> &Self::Target {
        &self.raw_id
    }
}

#[derive(Debug)]
pub struct DatByZone<T: DatFormat> {
    pub map: BTreeMap<ZoneId, Dat<T>>,
}

impl<T: DatFormat> Default for DatByZone<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T: DatFormat> DatByZone<T> {
    pub fn insert(&mut self, zone_id: usize, dat_id_value: usize) {
        self.map
            .insert(zone_id as ZoneId, (dat_id_value as u32).into());
    }

    pub fn get(&self, zone_id: &ZoneId) -> Option<&Dat<T>> {
        self.map.get(zone_id)
    }

    pub fn get_result(&self, zone_id: &ZoneId) -> Result<&Dat<T>> {
        self.map
            .get(zone_id)
            .ok_or(anyhow!("Did not find DAT for zone with ID {}.", zone_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatPath {
    pub rom_id: u8,
    pub folder_id: u16,
    pub file_id: u16,
}

impl DatPath {
    pub fn from_path(path: &PathBuf) -> Result<Self> {
        let mut path_iter = path.iter().rev();

        let file_part = path_iter.next().unwrap().to_str().unwrap();
        let folder_part = path_iter.next().unwrap().to_str().unwrap();
        let rom_part = path_iter.next().unwrap().to_str().unwrap();

        Ok(Self {
            rom_id: if rom_part == "ROM" {
                1
            } else {
                rom_part[3..].parse()?
            },
            folder_id: folder_part.parse()?,
            file_id: file_part[..file_part.len() - 4].parse()?,
        })
    }

    pub fn to_path(&self) -> PathBuf {
        let dat_path_buf: PathBuf = [
            if self.rom_id == 1 {
                "ROM".to_string()
            } else {
                format!("ROM{}", self.rom_id)
            },
            self.folder_id.to_string(),
            format!("{}.DAT", self.file_id),
        ]
        .into_iter()
        .collect();

        dat_path_buf
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatError {
    #[error("Could not find DAT for {0:?}")]
    DatIdNotFound(DatId),

    #[error("Could not DAT location for {0:?}")]
    DatMappingNotFound(DatId),

    #[error("Failed to load data for {0:?}: {1}")]
    DatLoadFailed(DatId, anyhow::Error),
}
