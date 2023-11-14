use anyhow::{anyhow, Result};
use std::{path::PathBuf, sync::Arc};

use dats::{
    base::{Dat, ZoneId},
    context::DatContext,
    dat_format::DatFormat,
    id_mapping::DatIdMapping,
};
use serde::{Deserialize, Serialize};

use crate::converters::{DatToYamlConverter, YamlToDatConverter};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, specta::Type, Serialize, Deserialize,
)]
#[serde(tag = "type", content = "index")]
pub enum DatDescriptor {
    DataMenu,

    // String tables
    AbilityNames,
    AbilityDescriptions,
    AreaNames,
    AreaNamesAlt,
    CharacterSelect,
    ChatFilterTypes,
    DayNames,
    Directions,
    EquipmentLocations,
    ErrorMessages,
    IngameMessages1,
    IngameMessages2,
    JobNames,
    KeyItems,
    MenuItemsDescription,
    MenuItemsText,
    MoonPhases,
    PolMessages,
    RaceNames,
    RegionNames,
    SpellNames,
    SpellDescriptions,
    StatusInfo,
    StatusNames,
    TimeAndPronouns,
    Titles,
    Misc1,
    Misc2,
    WeatherTypes,

    // Item data
    Armor,
    Armor2,
    Currency,
    GeneralItems,
    GeneralItems2,
    PuppetItems,
    UsableItems,
    Weapons,
    VouchersAndSlips,
    Monipulator,
    Instincts,

    // Global dialog
    MonsterSkillNames,
    StatusNamesDialog,
    EmoteMessages,
    SystemMessages1,
    SystemMessages2,
    SystemMessages3,
    SystemMessages4,
    UnityDialogs,

    // Dats by zone
    EntityNames(ZoneId),
    Dialog(ZoneId),
    Dialog2(ZoneId),
}

pub trait DatUsage {
    fn use_dat<T: DatFormat + Serialize + for<'a> serde::Deserialize<'a>>(
        self,
        dat: Dat<T>,
    ) -> Result<PathBuf>;
}

impl DatDescriptor {
    pub fn dat_to_yaml(
        &self,
        dat_context: Arc<DatContext>,
        raw_data_root_path: PathBuf,
    ) -> Result<PathBuf> {
        let data_path = raw_data_root_path.join(self.get_relative_path(&dat_context)? + ".yml");
        self.convert_with(DatToYamlConverter {
            dat_context,
            raw_data_path: data_path,
        })
    }

    pub fn yaml_to_dat(
        &self,
        dat_context: Arc<DatContext>,
        raw_data_root_path: PathBuf,
        dat_root_path: PathBuf,
    ) -> Result<PathBuf> {
        let raw_data_path = raw_data_root_path.join(self.get_relative_path(&dat_context)? + ".yml");
        self.convert_with(YamlToDatConverter {
            dat_context,
            raw_data_path,
            dat_root_path,
        })
    }

    fn get_zoned_file_name(
        dat_context: &DatContext,
        dir_name: &'static str,
        zone_id: &u16,
    ) -> Result<String> {
        Ok(format!(
            "{}/{}",
            dir_name,
            dat_context
                .zone_id_to_name
                .get(&zone_id)
                .ok_or(anyhow!("No zone name found for zone ID."))?
                .file_name
        ))
    }

    fn get_relative_path(&self, dat_context: &DatContext) -> Result<String> {
        match self {
            DatDescriptor::DataMenu => Ok("data_menu".to_string()),

            DatDescriptor::AbilityNames => Ok("ability_names".to_string()),
            DatDescriptor::AbilityDescriptions => Ok("ability_descriptions".to_string()),
            DatDescriptor::AreaNames => Ok("area_names".to_string()),
            DatDescriptor::AreaNamesAlt => Ok("area_names_alt".to_string()),
            DatDescriptor::CharacterSelect => Ok("character_select".to_string()),
            DatDescriptor::ChatFilterTypes => Ok("chat_filter_types".to_string()),
            DatDescriptor::DayNames => Ok("day_names".to_string()),
            DatDescriptor::Directions => Ok("directions".to_string()),
            DatDescriptor::EquipmentLocations => Ok("equipment_locations".to_string()),
            DatDescriptor::ErrorMessages => Ok("error_messages".to_string()),
            DatDescriptor::IngameMessages1 => Ok("ingame_messages1".to_string()),
            DatDescriptor::IngameMessages2 => Ok("ingame_messages2".to_string()),
            DatDescriptor::JobNames => Ok("job_names".to_string()),
            DatDescriptor::KeyItems => Ok("key_items".to_string()),
            DatDescriptor::MenuItemsDescription => Ok("menu_items_description".to_string()),
            DatDescriptor::MenuItemsText => Ok("menu_items_text".to_string()),
            DatDescriptor::MoonPhases => Ok("moon_phases".to_string()),
            DatDescriptor::PolMessages => Ok("pol_messages".to_string()),
            DatDescriptor::RaceNames => Ok("race_names".to_string()),
            DatDescriptor::RegionNames => Ok("region_names".to_string()),
            DatDescriptor::SpellNames => Ok("spell_names".to_string()),
            DatDescriptor::SpellDescriptions => Ok("spell_descriptions".to_string()),
            DatDescriptor::StatusInfo => Ok("status_info".to_string()),
            DatDescriptor::StatusNames => Ok("status_names".to_string()),
            DatDescriptor::TimeAndPronouns => Ok("time_and_pronouns".to_string()),
            DatDescriptor::Titles => Ok("titles".to_string()),
            DatDescriptor::Misc1 => Ok("misc1".to_string()),
            DatDescriptor::Misc2 => Ok("misc2".to_string()),
            DatDescriptor::WeatherTypes => Ok("weather_types".to_string()),

            DatDescriptor::Armor => Ok("items/armor".to_string()),
            DatDescriptor::Armor2 => Ok("items/armor2".to_string()),
            DatDescriptor::Currency => Ok("items/currency".to_string()),
            DatDescriptor::GeneralItems => Ok("items/general_items".to_string()),
            DatDescriptor::GeneralItems2 => Ok("items/general_items2".to_string()),
            DatDescriptor::PuppetItems => Ok("items/puppet_items".to_string()),
            DatDescriptor::UsableItems => Ok("items/usable_items".to_string()),
            DatDescriptor::Weapons => Ok("items/weapons".to_string()),
            DatDescriptor::VouchersAndSlips => Ok("items/vouchers_and_slips".to_string()),
            DatDescriptor::Monipulator => Ok("items/monipulator".to_string()),
            DatDescriptor::Instincts => Ok("items/instincts".to_string()),

            DatDescriptor::MonsterSkillNames => Ok("global_dialog/monster_skill_names".to_string()),
            DatDescriptor::StatusNamesDialog => Ok("global_dialog/status_names_dialog".to_string()),
            DatDescriptor::EmoteMessages => Ok("global_dialog/emote_messages".to_string()),
            DatDescriptor::SystemMessages1 => Ok("global_dialog/system_messages1".to_string()),
            DatDescriptor::SystemMessages2 => Ok("global_dialog/system_messages2".to_string()),
            DatDescriptor::SystemMessages3 => Ok("global_dialog/system_messages3".to_string()),
            DatDescriptor::SystemMessages4 => Ok("global_dialog/system_messages4".to_string()),
            DatDescriptor::UnityDialogs => Ok("global_dialog/unity_dialogs".to_string()),

            DatDescriptor::EntityNames(zone_id) => {
                Self::get_zoned_file_name(dat_context, "entity_names", zone_id)
            }
            DatDescriptor::Dialog(zone_id) => {
                Self::get_zoned_file_name(dat_context, "dialog", zone_id)
            }
            DatDescriptor::Dialog2(zone_id) => {
                Self::get_zoned_file_name(dat_context, "dialog2", zone_id)
            }
        }
    }

    fn get_zone_id(zone_dir_name: &str, dat_context: &DatContext) -> Option<ZoneId> {
        dat_context.zone_name_to_id_map.get(zone_dir_name).copied()
    }

    pub fn from_path(
        path: &PathBuf,
        raw_data_dir: &PathBuf,
        dat_context: &DatContext,
    ) -> Option<Self> {
        let path = path.strip_prefix(raw_data_dir).unwrap_or(path);

        let file_name = path
            .file_name()
            .and_then(|osstr| osstr.to_str())
            .map(|s| s.trim_end_matches(".yml"))?;

        if let Some(parent) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|osstr| osstr.to_str())
        {
            // Files in sub-directories
            return match parent {
                "entity_names" => {
                    Self::get_zone_id(file_name, dat_context).map(DatDescriptor::EntityNames)
                }
                "dialog" => Self::get_zone_id(file_name, dat_context).map(DatDescriptor::Dialog),
                "dialog2" => Self::get_zone_id(file_name, dat_context).map(DatDescriptor::Dialog2),

                "items" => match file_name {
                    "armor" => Some(DatDescriptor::Armor),
                    "armor2" => Some(DatDescriptor::Armor2),
                    "currency" => Some(DatDescriptor::Currency),
                    "general_items" => Some(DatDescriptor::GeneralItems),
                    "general_items2" => Some(DatDescriptor::GeneralItems2),
                    "puppet_items" => Some(DatDescriptor::PuppetItems),
                    "usable_items" => Some(DatDescriptor::UsableItems),
                    "weapons" => Some(DatDescriptor::Weapons),
                    "vouchers_and_slips" => Some(DatDescriptor::VouchersAndSlips),
                    "monipulator" => Some(DatDescriptor::Monipulator),
                    "instincts" => Some(DatDescriptor::Instincts),
                    _ => None,
                },
                "global_dialog" => match file_name {
                    "monster_skill_names" => Some(DatDescriptor::MonsterSkillNames),
                    "status_names_dialog" => Some(DatDescriptor::StatusNamesDialog),
                    "emote_messages" => Some(DatDescriptor::EmoteMessages),
                    "system_messages1" => Some(DatDescriptor::SystemMessages1),
                    "system_messages2" => Some(DatDescriptor::SystemMessages2),
                    "system_messages3" => Some(DatDescriptor::SystemMessages3),
                    "system_messages4" => Some(DatDescriptor::SystemMessages4),
                    "unity_dialogs" => Some(DatDescriptor::UnityDialogs),
                    _ => None,
                },
                _ => {
                    println!("Parent is: {}", parent);
                    None
                }
            };
        }

        // Files in root directory
        match file_name {
            "data_menu" => Some(DatDescriptor::DataMenu),

            "ability_names" => Some(DatDescriptor::AbilityNames),
            "ability_descriptions" => Some(DatDescriptor::AbilityDescriptions),
            "area_names" => Some(DatDescriptor::AreaNames),
            "area_names_alt" => Some(DatDescriptor::AreaNamesAlt),
            "character_select" => Some(DatDescriptor::CharacterSelect),
            "chat_filter_types" => Some(DatDescriptor::ChatFilterTypes),
            "day_names" => Some(DatDescriptor::DayNames),
            "directions" => Some(DatDescriptor::Directions),
            "equipment_locations" => Some(DatDescriptor::EquipmentLocations),
            "error_messages" => Some(DatDescriptor::ErrorMessages),
            "ingame_messages1" => Some(DatDescriptor::IngameMessages1),
            "ingame_messages2" => Some(DatDescriptor::IngameMessages2),
            "job_names" => Some(DatDescriptor::JobNames),
            "key_items" => Some(DatDescriptor::KeyItems),
            "menu" => Some(DatDescriptor::DataMenu),
            "menu_items_description" => Some(DatDescriptor::MenuItemsDescription),
            "menu_items_text" => Some(DatDescriptor::MenuItemsText),
            "moon_phases" => Some(DatDescriptor::MoonPhases),
            "pol_messages" => Some(DatDescriptor::PolMessages),
            "race_names" => Some(DatDescriptor::RaceNames),
            "region_names" => Some(DatDescriptor::RegionNames),
            "spell_names" => Some(DatDescriptor::SpellNames),
            "spell_descriptions" => Some(DatDescriptor::SpellDescriptions),
            "status_info" => Some(DatDescriptor::StatusInfo),
            "status_names" => Some(DatDescriptor::StatusNames),
            "time_and_pronouns" => Some(DatDescriptor::TimeAndPronouns),
            "titles" => Some(DatDescriptor::Titles),
            "misc1" => Some(DatDescriptor::Misc1),
            "misc2" => Some(DatDescriptor::Misc2),
            "weather_types" => Some(DatDescriptor::WeatherTypes),

            _ => None,
        }
    }

    fn convert_with<T: DatUsage>(self, converter: T) -> Result<PathBuf> {
        match self {
            DatDescriptor::DataMenu => converter.use_dat(DatIdMapping::get().data_menu.clone()),

            DatDescriptor::AbilityNames => {
                converter.use_dat(DatIdMapping::get().ability_names.clone())
            }
            DatDescriptor::AbilityDescriptions => {
                converter.use_dat(DatIdMapping::get().ability_descriptions.clone())
            }
            DatDescriptor::AreaNames => converter.use_dat(DatIdMapping::get().area_names.clone()),
            DatDescriptor::AreaNamesAlt => {
                converter.use_dat(DatIdMapping::get().area_names_alt.clone())
            }
            DatDescriptor::CharacterSelect => {
                converter.use_dat(DatIdMapping::get().character_select.clone())
            }
            DatDescriptor::ChatFilterTypes => {
                converter.use_dat(DatIdMapping::get().chat_filter_types.clone())
            }
            DatDescriptor::DayNames => converter.use_dat(DatIdMapping::get().day_names.clone()),
            DatDescriptor::Directions => converter.use_dat(DatIdMapping::get().directions.clone()),
            DatDescriptor::EquipmentLocations => {
                converter.use_dat(DatIdMapping::get().equipment_locations.clone())
            }
            DatDescriptor::ErrorMessages => {
                converter.use_dat(DatIdMapping::get().error_messages.clone())
            }
            DatDescriptor::IngameMessages1 => {
                converter.use_dat(DatIdMapping::get().ingame_messages_1.clone())
            }
            DatDescriptor::IngameMessages2 => {
                converter.use_dat(DatIdMapping::get().ingame_messages_2.clone())
            }
            DatDescriptor::JobNames => converter.use_dat(DatIdMapping::get().job_names.clone()),
            DatDescriptor::KeyItems => converter.use_dat(DatIdMapping::get().key_items.clone()),
            DatDescriptor::MenuItemsDescription => {
                converter.use_dat(DatIdMapping::get().menu_items_description.clone())
            }
            DatDescriptor::MenuItemsText => {
                converter.use_dat(DatIdMapping::get().menu_items_text.clone())
            }
            DatDescriptor::MoonPhases => converter.use_dat(DatIdMapping::get().moon_phases.clone()),
            DatDescriptor::PolMessages => {
                converter.use_dat(DatIdMapping::get().pol_messages.clone())
            }
            DatDescriptor::RaceNames => converter.use_dat(DatIdMapping::get().race_names.clone()),
            DatDescriptor::RegionNames => {
                converter.use_dat(DatIdMapping::get().region_names.clone())
            }
            DatDescriptor::SpellNames => converter.use_dat(DatIdMapping::get().spell_names.clone()),
            DatDescriptor::SpellDescriptions => {
                converter.use_dat(DatIdMapping::get().spell_descriptions.clone())
            }
            DatDescriptor::StatusInfo => converter.use_dat(DatIdMapping::get().status_info.clone()),
            DatDescriptor::StatusNames => {
                converter.use_dat(DatIdMapping::get().status_names.clone())
            }
            DatDescriptor::TimeAndPronouns => {
                converter.use_dat(DatIdMapping::get().time_and_pronouns.clone())
            }
            DatDescriptor::Titles => converter.use_dat(DatIdMapping::get().titles.clone()),
            DatDescriptor::Misc1 => converter.use_dat(DatIdMapping::get().misc1.clone()),
            DatDescriptor::Misc2 => converter.use_dat(DatIdMapping::get().misc2.clone()),
            DatDescriptor::WeatherTypes => {
                converter.use_dat(DatIdMapping::get().weather_types.clone())
            }

            DatDescriptor::Armor => converter.use_dat(DatIdMapping::get().armor.clone()),
            DatDescriptor::Armor2 => converter.use_dat(DatIdMapping::get().armor2.clone()),
            DatDescriptor::Currency => converter.use_dat(DatIdMapping::get().currency.clone()),
            DatDescriptor::GeneralItems => {
                converter.use_dat(DatIdMapping::get().general_items.clone())
            }
            DatDescriptor::GeneralItems2 => {
                converter.use_dat(DatIdMapping::get().general_items2.clone())
            }
            DatDescriptor::PuppetItems => {
                converter.use_dat(DatIdMapping::get().puppet_items.clone())
            }
            DatDescriptor::UsableItems => {
                converter.use_dat(DatIdMapping::get().usable_items.clone())
            }
            DatDescriptor::Weapons => converter.use_dat(DatIdMapping::get().weapons.clone()),
            DatDescriptor::VouchersAndSlips => {
                converter.use_dat(DatIdMapping::get().vouchers_and_slips.clone())
            }
            DatDescriptor::Monipulator => {
                converter.use_dat(DatIdMapping::get().monipulator.clone())
            }
            DatDescriptor::Instincts => converter.use_dat(DatIdMapping::get().instincts.clone()),

            // Global dialog
            DatDescriptor::MonsterSkillNames => {
                converter.use_dat(DatIdMapping::get().monster_skill_names.clone())
            }
            DatDescriptor::StatusNamesDialog => {
                converter.use_dat(DatIdMapping::get().status_names_dialog.clone())
            }
            DatDescriptor::EmoteMessages => {
                converter.use_dat(DatIdMapping::get().emote_messages.clone())
            }
            DatDescriptor::SystemMessages1 => {
                converter.use_dat(DatIdMapping::get().system_messages_1.clone())
            }
            DatDescriptor::SystemMessages2 => {
                converter.use_dat(DatIdMapping::get().system_messages_2.clone())
            }
            DatDescriptor::SystemMessages3 => {
                converter.use_dat(DatIdMapping::get().system_messages_3.clone())
            }
            DatDescriptor::SystemMessages4 => {
                converter.use_dat(DatIdMapping::get().system_messages_4.clone())
            }
            DatDescriptor::UnityDialogs => {
                converter.use_dat(DatIdMapping::get().unity_dialogs.clone())
            }

            // By zone
            DatDescriptor::EntityNames(zone_id) => {
                converter.use_dat(DatIdMapping::get().entities.get_result(&zone_id)?.clone())
            }
            DatDescriptor::Dialog(zone_id) => {
                converter.use_dat(DatIdMapping::get().dialog.get_result(&zone_id)?.clone())
            }
            DatDescriptor::Dialog2(zone_id) => {
                converter.use_dat(DatIdMapping::get().dialog.get_result(&zone_id)?.clone())
            }
        }
    }
}
