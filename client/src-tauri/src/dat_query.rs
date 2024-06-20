use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use dats::{
    base::{DatByZone, ZoneId},
    context::DatContext,
    dat_format::DatFormat,
    id_mapping::DatIdMapping,
};
use processor::dat_descriptor::DatDescriptor;
use serde::Serialize;
use tauri::async_runtime;

use crate::errors::AppError;

pub fn get_misc_dats() -> Vec<DatDescriptor> {
    vec![DatDescriptor::DataMenu]
}

pub fn get_standalone_string_dats() -> Vec<DatDescriptor> {
    vec![
        DatDescriptor::AbilityNames,
        DatDescriptor::AbilityDescriptions,
        DatDescriptor::AreaNames,
        DatDescriptor::AreaNamesAlt,
        DatDescriptor::CharacterSelect,
        DatDescriptor::ChatFilterTypes,
        DatDescriptor::DayNames,
        DatDescriptor::Directions,
        DatDescriptor::EquipmentLocations,
        DatDescriptor::ErrorMessages,
        DatDescriptor::IngameMessages1,
        // DatDescriptor::IngameMessages2, // TODO: XiStringTable parsing isn't fully supported yet
        DatDescriptor::JobNames,
        DatDescriptor::KeyItems,
        DatDescriptor::MenuItemsDescription,
        DatDescriptor::MenuItemsText,
        DatDescriptor::MoonPhases,
        // DatDescriptor::PolMessages,  // TODO: XiStringTable parsing isn't fully supported yet
        DatDescriptor::RaceNames,
        DatDescriptor::RegionNames,
        DatDescriptor::SpellNames,
        DatDescriptor::SpellDescriptions,
        DatDescriptor::StatusInfo,
        DatDescriptor::StatusNames,
        // DatDescriptor::TimeAndPronouns,  // TODO: XiStringTable parsing isn't fully supported yet
        DatDescriptor::Titles,
        DatDescriptor::Misc1,
        DatDescriptor::Misc2,
        DatDescriptor::WeatherTypes,
    ]
}

pub fn get_item_dats() -> Vec<DatDescriptor> {
    vec![
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
    ]
}

pub fn get_global_dialog_dats() -> Vec<DatDescriptor> {
    vec![
        DatDescriptor::MonsterSkillNames,
        DatDescriptor::StatusNamesDialog,
        DatDescriptor::EmoteMessages,
        DatDescriptor::SystemMessages1,
        DatDescriptor::SystemMessages2,
        DatDescriptor::SystemMessages3,
        DatDescriptor::SystemMessages4,
        DatDescriptor::UnityDialogs,
    ]
}

#[derive(Serialize, specta::Type)]
pub struct ZoneInfo {
    id: ZoneId,
    name: String,
}

async fn get_zone_ids_from_dats<T: DatFormat + 'static>(
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
                    .ok_or(anyhow!("No zone name for ID."))?;

                if dat_context.check_dat(&dat_id).is_ok() {
                    Ok::<_, AppError>(ZoneInfo {
                        id: zone_id.clone(),
                        name: zone_name.display_name.clone(),
                    })
                } else {
                    Err(anyhow!("DAT did not match type."))?
                }
            }))
        })
        .collect::<Vec<_>>();

    futures::future::join_all(handles)
        .await
        .into_iter()
        .flatten()
        .filter_map(|res| res.ok())
        .collect()
}

pub async fn get_zone_ids_for_type(
    dat_descriptor: DatDescriptor,
    dat_context: Arc<DatContext>,
) -> Vec<ZoneInfo> {
    match dat_descriptor {
        DatDescriptor::EntityNames(_) => {
            get_zone_ids_from_dats(&DatIdMapping::get().entities, dat_context).await
        }
        DatDescriptor::Dialog(_) => {
            get_zone_ids_from_dats(&DatIdMapping::get().dialog, dat_context).await
        }
        DatDescriptor::Dialog2(_) => {
            get_zone_ids_from_dats(&DatIdMapping::get().dialog2, dat_context).await
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
