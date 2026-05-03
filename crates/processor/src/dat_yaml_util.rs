use anyhow::{Result, anyhow};
use std::{path::PathBuf, sync::Arc};

use dats::{
    base::ZoneId,
    context::DatContext,
    id_mapping::{DatDescriptor, DatLanguage, DatWithLang},
};

use crate::converters::{DatToYamlConverter, YamlToDatConverter};

pub struct DatYamlUtil;

macro_rules! define_dat_files {
    (
        // Simple mappings (variant => path)
        simple: {
            $($variant:ident => $path:literal),* $(,)?
        }
        // Directory-based mappings (dir => (variant => sub_path))
        $(, dirs: {
            $($dir:literal: {
                $($dir_variant:ident => $dir_path:literal),* $(,)?
            }),* $(,)?
        })?
        // Zone-based mappings (variant => directory)
        $(, zones: {
            $($zone_variant:ident => $zone_dir:literal),* $(,)?
        })?
    ) => {
        impl DatYamlUtil {
            fn get_relative_path(dat_descriptor: &DatDescriptor, dat_context: &DatContext) -> Result<String> {
                match dat_descriptor {
                    $(DatDescriptor::$variant => Ok($path.to_string()),)*
                    $($($(DatDescriptor::$dir_variant => Ok(concat!($dir, "/", $dir_path).to_string()),)*)*)?
                    $($(
                        DatDescriptor::$zone_variant(zone_id) => {
                            Self::get_zoned_file_name(dat_context, $zone_dir, zone_id)
                        }
                    )*)?
                }
            }

            pub fn dat_from_path(
                path: &PathBuf,
                raw_data_dir: &PathBuf,
                dat_context: &DatContext,
            ) -> Option<DatWithLang> {
                let path = path.strip_prefix(raw_data_dir).unwrap_or(path);

                let mut file_name = path
                    .file_name()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.trim_end_matches(".yml"))?;

                let lang = if file_name.ends_with("_jp") {
                    file_name = file_name.trim_end_matches("_jp");
                    DatLanguage::Japanese
                } else {
                    DatLanguage::English
                };

                let descriptor = if let Some(parent) = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|osstr| osstr.to_str())
                {
                    // Files in sub-directories
                    match parent {
                        $($(
                            $dir => match file_name {
                                $($dir_path => Some(DatDescriptor::$dir_variant),)*
                                _ => None,
                            },
                        )*)?
                        $($(
                            $zone_dir => Self::get_zone_id(file_name, dat_context).map(DatDescriptor::$zone_variant),
                        )*)?
                        _ => {
                            None
                        }
                    }
                }
                else
                {
                    // Files in root directory
                    match file_name {
                        $($path => Some(DatDescriptor::$variant),)*
                        _ => None,
                    }
                };

                Some(DatWithLang::new(descriptor?, lang))
            }
        }
    };
}

define_dat_files! {
    simple: {
        DataMenu => "data_menu",
        QuestsMissionsKeyItems => "quests_missions_keyitems",
        MeritTable => "merit_table",
        MeritCategoryTable => "merit_category_table",

        // String tables
        AbilityNames => "ability_names",
        AbilityDescriptions => "ability_descriptions",
        AutoTranslate => "auto_translate",
        AreaNames => "area_names",
        AreaNamesShort => "area_names_short",
        AreaNamesAlt => "area_names_alt",
        Augments => "augments",
        BlueMagic => "blue_magic",
        CallMount => "call_mount",
        CharacterSelect => "character_select",
        ChatFilterTypes => "chat_filter_types",
        ChocoboNames => "chocobo_names",
        CommandUsage => "command_usage",
        DayNames => "day_names",
        Directions => "directions",
        EinherjarChambers => "einherjar_chambers",
        Emotes => "emotes",
        EquipmentLocations => "equipment_locations",
        EquipmentLocationsAlt => "equipment_locations_alt",
        ErrorMessages => "error_messages",
        IngameMessages1 => "ingame_messages_1",
        IngameMessages2 => "ingame_messages_2",
        JobNames => "job_names",
        JobNamesShort => "job_names_short",
        JobPointBonuses => "job_point_bonuses",
        JobPointGifts => "job_point_gifts",
        KeyItems => "key_items",
        MenuItemsDescription => "menu_items_description",
        MenuItemsText => "menu_items_text",
        Merits => "merits",
        MoblinMazeMongers => "moblin_maze_mongers",
        Modifiers => "modifiers",
        MonsterFamilies => "monster_families",
        MoonPhases => "moon_phases",
        MountNames => "mount_names",
        PankrationNames => "pankration_names",
        PolMessages => "pol_messages",
        RaceNames => "race_names",
        RegionNames => "region_names",
        ServerNames => "server_names",
        SpellNames => "spell_names",
        SpellDescriptions => "spell_descriptions",
        StatusInfo => "status_info",
        StatusNames => "status_names",
        TimeAndPronouns => "time_and_pronouns",
        Titles => "titles",
        TrustMessages => "trust_messages",
        Misc1 => "misc1",
        Misc2 => "misc2",
        WeatherTypes => "weather_types",
    },
    dirs: {
        // Item data
        "items": {
            Armor => "armor",
            Armor2 => "armor2",
            Currency => "currency",
            GeneralItems => "general_items",
            GeneralItems2 => "general_items2",
            PuppetItems => "puppet_items",
            UsableItems => "usable_items",
            Weapons => "weapons",
            VouchersAndSlips => "vouchers_and_slips",
            Monipulator => "monipulator",
            Instincts => "instincts",
            Furniture => "furniture",
        },

        // Global dialog
        "global_dialog": {
            MonsterSkillNames => "monster_skill_names",
            StatusNamesDialog => "status_names_dialog",
            EmoteMessages => "emote_messages",
            SystemMessages1 => "system_messages_1",
            SystemMessages2 => "system_messages_2",
            SystemMessages3 => "system_messages_3",
            SystemMessages4 => "system_messages_4",
            UnityDialogs => "unity_dialogs",
        },

        // Missions
        "missions": {
            MissionsAcp => "acp",
            MissionsAmke => "amke",
            MissionsAsa => "asa",
            MissionsAssault => "assault",
            MissionsBastok => "bastok",
            MissionsCampaign => "campaign",
            MissionsCop => "cop",
            MissionsRov => "rov",
            MissionsSandoria => "sandoria",
            MissionsSoa => "soa",
            MissionsToau => "toau",
            MissionsWindurst => "windurst",
            MissionsWotg => "wotg",
            MissionsZilart => "zilart",
        },

        // Quests
        "quests": {
            QuestsAbyssea => "abyssea",
            QuestsBastok => "bastok",
            QuestsCoalition => "coalition",
            QuestsJeuno => "jeuno",
            QuestsOther => "other",
            QuestsOutlands => "outlands",
            QuestsSandoria => "sandoria",
            QuestsSoa => "soa",
            QuestsToau => "toau",
            QuestsWindurst => "windurst",
            QuestsWotg => "wotg",
        },
    },
    zones: {
        ZoneData => "zone_data",
        EntityNames => "entity_names",
        Dialog => "dialog",
        Dialog2 => "dialog2",
        Events => "events",
    }
}

impl DatYamlUtil {
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

    fn get_zone_id(zone_dir_name: &str, dat_context: &DatContext) -> Option<ZoneId> {
        dat_context.zone_name_to_id_map.get(zone_dir_name).copied()
    }

    pub fn dat_to_yaml(
        dat_descriptor: &DatDescriptor,
        lang: DatLanguage,
        dat_context: Arc<DatContext>,
        raw_data_root_path: PathBuf,
    ) -> Result<PathBuf> {
        match lang {
            DatLanguage::English => {
                let data_path = raw_data_root_path
                    .join(Self::get_relative_path(dat_descriptor, &dat_context)? + ".yml");

                dat_descriptor.use_dat_with(DatToYamlConverter {
                    dat_context,
                    raw_data_path: data_path,
                })
            }
            DatLanguage::Japanese => {
                let data_path = raw_data_root_path
                    .join(Self::get_relative_path(dat_descriptor, &dat_context)? + "_jp.yml");

                dat_descriptor.use_jp_dat_with(DatToYamlConverter {
                    dat_context,
                    raw_data_path: data_path,
                })
            }
        }
    }

    pub fn yaml_to_dat(
        dat_descriptor: &DatDescriptor,
        lang: DatLanguage,
        dat_context: Arc<DatContext>,
        raw_data_root_path: PathBuf,
        dat_root_path: PathBuf,
    ) -> Result<PathBuf> {
        match lang {
            DatLanguage::English => {
                let raw_data_path = raw_data_root_path
                    .join(Self::get_relative_path(dat_descriptor, &dat_context)? + ".yml");

                dat_descriptor.use_dat_with(YamlToDatConverter {
                    dat_context,
                    raw_data_path,
                    dat_root_path,
                })
            }
            DatLanguage::Japanese => {
                let raw_data_path = raw_data_root_path
                    .join(Self::get_relative_path(dat_descriptor, &dat_context)? + "_jp.yml");

                dat_descriptor.use_jp_dat_with(YamlToDatConverter {
                    dat_context,
                    raw_data_path,
                    dat_root_path,
                })
            }
        }
    }
}
