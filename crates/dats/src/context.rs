use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::{
    base::{Dat, DatError, DatId, DatPath, ZoneId},
    dat_format::DatFormat,
    formats::dmsg2_string_table::Dmsg2Content,
    id_mapping::DatIdMapping,
    sanitize_filename::sanitize_filename,
};
use anyhow::{anyhow, Result};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DatContext {
    pub ffxi_path: PathBuf,
    pub id_map: HashMap<DatId, DatPath>,

    pub zone_name_to_id_map: HashMap<String, ZoneId>,
    pub zone_id_to_name: HashMap<ZoneId, ZoneName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneName {
    pub display_name: String,
    pub file_name: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedDat<T> {
    pub dat: T,
    pub path: PathBuf,
}

impl DatContext {
    pub fn from_ffxi_path(ffxi_path: PathBuf) -> Result<Self> {
        let mut context = Self::from_path_without_zone_mappings(ffxi_path)?;

        context.build_zone_mappings()?;

        Ok(context)
    }

    pub fn from_path_and_zone_mappings(
        ffxi_path: PathBuf,
        zone_id_to_name: HashMap<ZoneId, ZoneName>,
    ) -> Result<Self> {
        let mut context = Self::from_path_without_zone_mappings(ffxi_path)?;

        context.zone_id_to_name = zone_id_to_name;

        for (zone_id, zone_names) in &context.zone_id_to_name {
            context
                .zone_name_to_id_map
                .insert(zone_names.file_name.clone(), zone_id.clone());
        }

        Ok(context)
    }

    fn from_path_without_zone_mappings(mut ffxi_path: PathBuf) -> Result<Self> {
        ffxi_path = Self::find_ffxi_path(ffxi_path)?;

        let id_map = Self::build_rom_id_map(&ffxi_path)?;

        Ok(Self {
            ffxi_path,
            id_map,
            zone_name_to_id_map: Default::default(),
            zone_id_to_name: Default::default(),
        })
    }

    fn build_zone_mappings(&mut self) -> Result<()> {
        // Initialize the mappings between zone ID and name
        let zone_data = self.get_data_from_dat(&DatIdMapping::get().area_names)?;

        let mut previous_names = HashSet::new();
        for (zone_id, (_, zone_string_list)) in zone_data.dat.lists.into_iter().enumerate() {
            let display_content = zone_string_list
                .content
                .first()
                .ok_or_else(|| anyhow!("No string found for zone {}.", zone_id))?
                .clone();

            let mut display_name = match display_content {
                Dmsg2Content::String { string } => string,
                Dmsg2Content::Flags { .. } => {
                    return Err(anyhow!("Expected string content for zone name."))
                }
            };

            if display_name.is_empty() {
                display_name = format!("_unnamed_ID-{}", zone_id);
            } else if previous_names.contains(&display_name) {
                display_name = format!("{} ID-{}", display_name, zone_id);
            }
            previous_names.insert(display_name.clone());

            let zone_name = ZoneName {
                file_name: sanitize_filename(&display_name),
                display_name,
            };

            self.zone_id_to_name
                .insert(zone_id as u16, zone_name.clone());
            self.zone_name_to_id_map
                .insert(zone_name.file_name, zone_id as u16);
        }

        Ok(())
    }

    pub fn find_ffxi_path(mut ffxi_path: PathBuf) -> Result<PathBuf> {
        // If there's a VTABLE.DAT in this folder, it's assumed to be correct already.
        if ffxi_path.join("VTABLE.DAT").exists() {
            return Ok(ffxi_path);
        }

        match ffxi_path
            .iter()
            .last()
            .and_then(|part| part.to_str())
            .ok_or(anyhow!("Invalid path"))?
        {
            "FINAL FANTASY XI" => {
                // It should be the correct folder already
            }
            "SquareEnix" => {
                ffxi_path.push("FINAL FANTASY XI");
            }
            _ => {
                ffxi_path.push("SquareEnix");
                ffxi_path.push("FINAL FANTASY XI");
                if !ffxi_path.exists() {
                    return Err(anyhow!("Could not find a FFXI install at the given path."));
                }
            }
        };

        Ok(ffxi_path)
    }

    pub fn build_rom_id_map(ffxi_path: &PathBuf) -> Result<HashMap<DatId, DatPath>> {
        let mut id_map = HashMap::new();

        // Handle first non-numbered tables
        let vtable_dat_path = ffxi_path.join("VTABLE.DAT");
        let ftable_dat_path = ffxi_path.join("FTABLE.DAT");

        // Build up DAT map from lookup table files
        Self::insert_into_id_map(&mut id_map, 1, vtable_dat_path, ftable_dat_path)?;

        for rom_id in 2u8.. {
            let vtable_dat_path = ffxi_path.join(format!("ROM{}/VTABLE{}.DAT", rom_id, rom_id));
            let ftable_dat_path = ffxi_path.join(format!("ROM{}/FTABLE{}.DAT", rom_id, rom_id));
            if Self::insert_into_id_map(&mut id_map, rom_id, vtable_dat_path, ftable_dat_path)
                .is_err()
            {
                break;
            }
        }

        Ok(id_map)
    }

    fn insert_into_id_map(
        id_map: &mut HashMap<DatId, DatPath>,
        rom_id: u8,
        vtable_dat_path: PathBuf,
        ftable_dat_path: PathBuf,
    ) -> Result<()> {
        let mut vtable_data = Vec::new();
        File::open(&vtable_dat_path)
            .map_err(|_| {
                anyhow!(
                    "Could not open necessary file: {}",
                    vtable_dat_path.to_string_lossy()
                )
            })?
            .read_to_end(&mut vtable_data)?;

        let mut ftable_dat = File::open(&ftable_dat_path).map_err(|_| {
            anyhow!(
                "Could not open necessary file: {}",
                ftable_dat_path.to_string_lossy()
            )
        })?;

        let mut dat_path_buf = [0u8; 2];

        let _ = vtable_data
            .into_iter()
            .enumerate()
            .filter_map(|(byte_idx, byte)| {
                if byte == rom_id {
                    Some(byte_idx as u64)
                } else {
                    None
                }
            })
            .filter_map(|byte_idx| {
                ftable_dat.seek(SeekFrom::Start(byte_idx * 2u64)).ok()?;
                ftable_dat.read_exact(&mut dat_path_buf).ok()?;

                let combined_id = u16::from_le_bytes(dat_path_buf);
                id_map.insert(
                    DatId::from(byte_idx as u32),
                    DatPath {
                        rom_id,
                        folder_id: (combined_id >> 7) as u16,
                        file_id: (combined_id & 0x7F) as u16,
                    },
                );

                Some(())
            })
            .collect::<Vec<_>>();

        Ok(())
    }

    pub fn get_data_from_dat_id<T: DatFormat>(&self, id: DatId) -> Result<T, DatError> {
        T::from_path(&self.get_dat_path(id)?)
            .map_err(|err| DatError::DatLoadFailed(id.clone(), err))
    }

    pub fn get_data_from_dat<T: DatFormat>(
        &self,
        id: &Dat<T>,
    ) -> Result<ExtractedDat<T>, DatError> {
        let path = self.get_dat_path(id)?;
        T::from_path(&path)
            .map(|dat| ExtractedDat { dat, path })
            .map_err(|err| DatError::DatLoadFailed(id.into(), err))
    }

    pub fn check_dat<T: DatFormat>(&self, id: &Dat<T>) -> Result<(), DatError> {
        T::check_path(&self.get_dat_path(id)?)
            .map_err(|err| DatError::DatLoadFailed(id.into(), err))
    }

    pub fn get_data_from_dat_checked<T: DatFormat>(&self, id: &Dat<T>) -> Result<T, DatError> {
        T::from_path_checked(&self.get_dat_path(id)?)
            .map_err(|err| DatError::DatLoadFailed(id.into(), err))
    }

    pub fn get_dat_path(&self, id: impl Into<DatId>) -> Result<PathBuf, DatError> {
        id.into().get_ffxi_dat_path(self)
    }

    pub fn get_dat_id(&self, dat_path: DatPath) -> Option<DatId> {
        self.id_map.iter().find_map(|entry| {
            if entry.1 == &dat_path {
                Some(entry.0.clone())
            } else {
                None
            }
        })
    }
}
