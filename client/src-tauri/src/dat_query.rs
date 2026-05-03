use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use dats::{
    base::{DatByZone, ZoneId},
    context::DatContext,
    dat_format::DatFormat,
    formats::zone_data::zone_model::ZoneMesh,
    id_mapping::{DatDescriptor, DatIdMapping},
};
use serde::{Deserialize, Serialize};
use tauri::async_runtime;

use crate::errors::AppError;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, specta::Type,
)]
pub struct DatDescriptorInfo {
    descriptor: DatDescriptor,
    has_jp: bool,
}

const fn to_info(descriptor: DatDescriptor) -> DatDescriptorInfo {
    DatDescriptorInfo {
        descriptor,
        has_jp: descriptor.has_jp_dat(),
    }
}

const fn to_infos<const N: usize>(source: [DatDescriptor; N]) -> [DatDescriptorInfo; N] {
    let mut result: [DatDescriptorInfo; N] = [to_info(DatDescriptor::MonsterSkillNames); N];

    let mut i = 0;
    while i < N {
        result[i] = to_info(source[i]);
        i += 1;
    }

    result
}

pub static MISC_DATS: &'static [DatDescriptorInfo] = {
    &to_infos([
        DatDescriptor::DataMenu,
        DatDescriptor::QuestsMissionsKeyItems,
        DatDescriptor::MeritTable,
        DatDescriptor::MeritCategoryTable,
    ])
};

pub static STANDALONE_DATS: &'static [DatDescriptorInfo] = {
    &to_infos([
        DatDescriptor::AbilityNames,
        DatDescriptor::AbilityDescriptions,
        DatDescriptor::AreaNames,
        DatDescriptor::AreaNamesShort,
        DatDescriptor::AreaNamesAlt,
        DatDescriptor::Augments,
        DatDescriptor::AutoTranslate,
        DatDescriptor::BlueMagic,
        DatDescriptor::CallMount,
        DatDescriptor::CharacterSelect,
        DatDescriptor::ChatFilterTypes,
        DatDescriptor::ChocoboNames,
        DatDescriptor::CommandUsage,
        DatDescriptor::DayNames,
        DatDescriptor::Directions,
        DatDescriptor::EinherjarChambers,
        DatDescriptor::Emotes,
        DatDescriptor::EquipmentLocations,
        DatDescriptor::EquipmentLocationsAlt,
        DatDescriptor::ErrorMessages,
        DatDescriptor::IngameMessages1,
        // DatDescriptor::IngameMessages2, // TODO: XiStringTable parsing isn't fully supported yet
        DatDescriptor::JobNames,
        DatDescriptor::JobNamesShort,
        DatDescriptor::JobPointBonuses,
        DatDescriptor::JobPointGifts,
        DatDescriptor::KeyItems,
        DatDescriptor::MenuItemsDescription,
        DatDescriptor::MenuItemsText,
        DatDescriptor::Merits,
        DatDescriptor::MoblinMazeMongers,
        DatDescriptor::Modifiers,
        DatDescriptor::MonsterFamilies,
        DatDescriptor::MoonPhases,
        DatDescriptor::MountNames,
        DatDescriptor::PankrationNames,
        // DatDescriptor::PolMessages,  // TODO: XiStringTable parsing isn't fully supported yet
        DatDescriptor::RaceNames,
        DatDescriptor::RegionNames,
        DatDescriptor::ServerNames,
        DatDescriptor::SpellNames,
        DatDescriptor::SpellDescriptions,
        DatDescriptor::StatusInfo,
        DatDescriptor::StatusNames,
        // DatDescriptor::TimeAndPronouns,  // TODO: XiStringTable parsing isn't fully supported yet
        DatDescriptor::Titles,
        DatDescriptor::TrustMessages,
        DatDescriptor::Misc1,
        DatDescriptor::Misc2,
        DatDescriptor::WeatherTypes,
    ])
};

pub static MISSION_DATS: &'static [DatDescriptorInfo] = {
    &to_infos([
        DatDescriptor::MissionsAcp,
        DatDescriptor::MissionsAmke,
        DatDescriptor::MissionsAsa,
        DatDescriptor::MissionsAssault,
        DatDescriptor::MissionsBastok,
        DatDescriptor::MissionsCampaign,
        DatDescriptor::MissionsCop,
        DatDescriptor::MissionsRov,
        DatDescriptor::MissionsSandoria,
        DatDescriptor::MissionsSoa,
        DatDescriptor::MissionsToau,
        DatDescriptor::MissionsWindurst,
        DatDescriptor::MissionsWotg,
        DatDescriptor::MissionsZilart,
    ])
};

pub static QUEST_DATS: &'static [DatDescriptorInfo] = {
    &to_infos([
        DatDescriptor::QuestsAbyssea,
        DatDescriptor::QuestsBastok,
        DatDescriptor::QuestsCoalition,
        DatDescriptor::QuestsJeuno,
        DatDescriptor::QuestsOther,
        DatDescriptor::QuestsOutlands,
        DatDescriptor::QuestsSandoria,
        DatDescriptor::QuestsSoa,
        DatDescriptor::QuestsToau,
        DatDescriptor::QuestsWindurst,
        DatDescriptor::QuestsWotg,
    ])
};

pub static ITEM_DATS: &'static [DatDescriptorInfo] = {
    &to_infos([
        DatDescriptor::Armor,
        DatDescriptor::Armor2,
        // DatDescriptor::Currency, // TODO: can't currently parse this
        DatDescriptor::GeneralItems,
        DatDescriptor::GeneralItems2,
        DatDescriptor::PuppetItems,
        DatDescriptor::UsableItems,
        DatDescriptor::Weapons,
        DatDescriptor::VouchersAndSlips,
        // DatDescriptor::Monipulator, // TODO: fields seems to be very different compared to other items
        DatDescriptor::Instincts,
        DatDescriptor::Furniture,
    ])
};

pub static GLOBAL_DIALOG_DATS: &'static [DatDescriptorInfo] = {
    &to_infos([
        DatDescriptor::MonsterSkillNames,
        DatDescriptor::StatusNamesDialog,
        DatDescriptor::EmoteMessages,
        DatDescriptor::SystemMessages1,
        DatDescriptor::SystemMessages2,
        DatDescriptor::SystemMessages3,
        DatDescriptor::SystemMessages4,
        DatDescriptor::UnityDialogs,
    ])
};

#[derive(Serialize, specta::Type)]
pub struct ZoneInfo {
    pub id: ZoneId,
    pub name: String,
    pub dat_path: String,
}

#[derive(Serialize, specta::Type)]
pub struct TriangleMetadata {
    // Triangle-specific
    pub grid_entry_idx: u32,
    pub mesh_entry_idx: u32,
    pub material: u8,
    pub is_invalid_triangle: bool,
    pub is_barrier: bool,

    // Placement
    pub o2w: [[f32; 3]; 4],
    pub o2w_opts: [[u16; 2]; 4],
    pub w2o: [[f32; 3]; 4],
    pub w2o_opts: [[u16; 2]; 4],
    pub unk_floats: [f32; 9],
    pub data_field_1: u32,
    pub data_field_2: u32,
    pub unk_bytes: [u8; 4],
    pub unk_1: u32,
    pub min_y: f32,
    pub max_y: f32,
    pub unk_2: u32,

    pub map_id: u8,

    // Block
    pub block_flags: u16,
}

pub async fn get_zone_model(zone_id: ZoneId, dat_context: Arc<DatContext>) -> Option<ZoneMesh> {
    let zone_data_dat = DatIdMapping::get().zone_data.get(&zone_id)?;

    let zone_data = dat_context.get_data_from_dat(zone_data_dat).ok()?;

    let zone_model = ZoneMesh::parse_from_zone_data(&zone_data.dat).ok()?;

    Some(zone_model.clone())
}

async fn get_zone_infos_from_dats<T: DatFormat + 'static>(
    dat_by_zone: &DatByZone<T>,
    dat_context: Arc<DatContext>,
) -> Vec<ZoneInfo> {
    let handles = dat_by_zone
        .map
        .iter()
        .filter_map(|(zone_id, dat_id)| {
            let zone_id = zone_id.clone();
            let dat_id = dat_id.clone();
            let dat_context = dat_context.clone();

            Some(async_runtime::spawn(async move {
                let zone_name = dat_context
                    .zone_id_to_name
                    .get(&zone_id)
                    .ok_or(anyhow!("No zone name for ID: {zone_id}."))?;

                match dat_context.check_dat(&dat_id) {
                    Ok(_) => Ok::<_, AppError>(ZoneInfo {
                        id: zone_id.clone(),
                        name: zone_name.display_name.clone(),
                        dat_path: dat_id
                            .get_relative_dat_path(&dat_context)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                    }),
                    Err(err) => Err(err.into()),
                }
            }))
        })
        .collect::<Vec<_>>();

    futures::future::join_all(handles)
        .await
        .into_iter()
        .flatten()
        .filter_map(|res| match res {
            Ok(res) => Some(res),
            Err(err) => {
                eprintln!("Failed to load DAT: {err}");
                None
            }
        })
        .collect()
}

pub async fn get_zone_infos_for_type(
    dat_descriptor: DatDescriptor,
    dat_context: Arc<DatContext>,
) -> Vec<ZoneInfo> {
    match dat_descriptor {
        DatDescriptor::ZoneData(_) => {
            get_zone_infos_from_dats(&DatIdMapping::get().zone_data, dat_context).await
        }
        DatDescriptor::EntityNames(_) => {
            get_zone_infos_from_dats(&DatIdMapping::get().entities, dat_context).await
        }
        DatDescriptor::Dialog(_) => {
            get_zone_infos_from_dats(&DatIdMapping::get().dialog, dat_context).await
        }
        DatDescriptor::Dialog2(_) => {
            get_zone_infos_from_dats(&DatIdMapping::get().dialog2, dat_context).await
        }
        DatDescriptor::Events(_) => {
            get_zone_infos_from_dats(&DatIdMapping::get().events, dat_context).await
        }
        _ => {
            vec![]
        }
    }
}

#[derive(Serialize, specta::Type)]
pub struct BrowseInfo {
    path: PathBuf,
    id: u32,
}

pub async fn get_browse_info(dat_context: Arc<DatContext>) -> Vec<BrowseInfo> {
    dat_context
        .id_map
        .iter()
        .map(|entry| BrowseInfo {
            path: entry.1.to_path(),
            id: entry.0.get_inner(),
        })
        .collect()
}
