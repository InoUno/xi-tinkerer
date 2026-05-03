use std::sync::OnceLock;

use crate::{
    base::{Dat, DatByZone, ZoneId},
    dat_format::DatFormat,
    formats::{
        auto_translate::AutoTranslate, dialog::Dialog, dmsg_table::DmsgTable,
        entity_names::EntityNames, events::Events, furniture_data::FurnitureData,
        item_info::ItemInfoTable, menu_table::MenuTable,
        merit_category_table::MeritCategoryTable, merit_table::MeritTable,
        status_info::StatusInfoTable, xistring_table::XiStringTable, zone_data::ZoneData,
    },
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum DatLanguage {
    #[default]
    English,
    Japanese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DatWithLang {
    pub descriptor: DatDescriptor,
    pub lang: DatLanguage,
}

impl DatWithLang {
    pub fn new(descriptor: DatDescriptor, lang: DatLanguage) -> Self {
        Self { descriptor, lang }
    }
}

impl From<DatDescriptor> for DatWithLang {
    fn from(descriptor: DatDescriptor) -> Self {
        DatWithLang {
            descriptor,
            lang: DatLanguage::English,
        }
    }
}

pub trait DatUsage<U> {
    fn use_dat<T: DatFormat + Serialize + for<'a> serde::Deserialize<'a>>(
        self,
        dat: Dat<T>,
    ) -> anyhow::Result<U>;
}

macro_rules! define_dat_mappings {
    (
        simple: {
            $($variant:ident => $dat_format:ident($dat_id_en:literal$(, $dat_id_jp:literal)?)),* $(,)?
        }
        $(, zones: {
            $($zone_variant:ident => $zone_dat_field:ident),* $(,)?
        })?
    ) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[cfg_attr(feature = "specta", derive(specta::Type))]
        #[serde(tag = "type", content = "index")]
        pub enum DatDescriptor {
            $($variant,)*
            $($($zone_variant(ZoneId),)*)?
        }

        impl DatDescriptor {
            pub fn use_dat_with<T: DatUsage<U>, U>(&self, dat_user: T) -> anyhow::Result<U> {
                match self {
                    $(Self::$variant => dat_user.use_dat(Dat::<$dat_format>::from($dat_id_en)),)*
                    $($(
                        Self::$zone_variant(zone_id) => {
                            dat_user.use_dat(DatIdMapping::get().$zone_dat_field.get_result(&zone_id)?.clone())
                        }
                    )*)?
                }
            }

            pub const fn has_jp_dat(&self) -> bool {
                match self {
                    $($(Self::$variant => $dat_id_jp > 0,)*)*
                    _ => false
                }
            }

            pub fn use_jp_dat_with<T: DatUsage<U>, U>(&self, dat_user: T) -> anyhow::Result<U> {
                match self {
                    $($(Self::$variant => dat_user.use_dat(Dat::<$dat_format>::from($dat_id_jp)),)*)*
                    _ => Err(anyhow::anyhow!("JP DAT not mapped."))
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct DatIdMapping {
    // Zone data
    pub zone_data: DatByZone<ZoneData>,
    pub entities: DatByZone<EntityNames>,
    pub dialog: DatByZone<Dialog>,
    pub dialog2: DatByZone<Dialog>,
    pub events: DatByZone<Events>,

    pub area_names: Dat<DmsgTable>,
}

static DAT_ID_MAPPING: OnceLock<DatIdMapping> = OnceLock::new();

impl DatIdMapping {
    pub fn get() -> &'static Self {
        DAT_ID_MAPPING.get_or_init(|| {
            // Zone data
            let mut zone_data = DatByZone::default();
            // Zones 0-255
            (0..256).into_iter().for_each(|idx| {
                zone_data.insert(idx, 100 + idx);
            });
            // Zones 256-512
            (0..256).into_iter().for_each(|idx| {
                zone_data.insert(idx + 256, 83891 + idx);
            });

            // Entities
            let mut entities = DatByZone::default();
            // Zones 1-255
            (0..256).into_iter().for_each(|idx| {
                entities.insert(idx, 6720 + idx);
            });
            // Zones 256-512
            (0..256).into_iter().for_each(|idx| {
                entities.insert(256 + idx, 86491 + idx);
            });
            // Zones 1000+
            (0..256).into_iter().for_each(|idx| {
                entities.insert(1000 + idx, 67911 + idx);
            });

            // Dialog text
            let mut dialog = DatByZone::default();
            // Zones 0-255
            (0..256).into_iter().for_each(|idx| {
                dialog.insert(idx, 6420 + idx);
            });
            // Zones 256-512
            (0..256).into_iter().for_each(|idx| {
                dialog.insert(idx + 256, 85590 + idx);
            });

            // Events
            // Zones 0-255
            let mut events = DatByZone::default();
            (0..256).into_iter().for_each(|idx| {
                events.insert(idx, 5820 + idx);
            });
            // Zones 256-512
            (0..256).into_iter().for_each(|idx| {
                events.insert(idx + 256, 84991 + idx);
            });

            // Secondary dialog text
            let mut dialog2 = DatByZone::default();
            // Just whitegate?
            dialog2.insert(50, 57945);

            Self {
                zone_data,
                entities,
                dialog,
                dialog2,
                events,

                area_names: 55465.into(),
            }
        })
    }
}

define_dat_mappings! {
    simple: {
        DataMenu => MenuTable(81),
        QuestsMissionsKeyItems => MenuTable(82),
        MeritTable => MeritTable(88),
        MeritCategoryTable => MeritCategoryTable(89),

        // String tables
        AbilityNames => DmsgTable(55701, 55581),
        AbilityDescriptions => DmsgTable(55733, 55613),
        AreaNames => DmsgTable(55465, 55535),
        AreaNamesShort => DmsgTable(55466),
        AreaNamesAlt => DmsgTable(55661),
        Augments => DmsgTable(55692, 55572),
        BlueMagic => DmsgTable(55685, 55565),
        CallMount => DmsgTable(55682, 55562),
        CharacterSelect => DmsgTable(55470),
        ChatFilterTypes => DmsgTable(55650, 55530),
        ChocoboNames => DmsgTable(55474),
        CommandUsage => DmsgTable(55687, 55567),
        DayNames => DmsgTable(55658, 55538),
        Directions => DmsgTable(55659, 55539),
        EinherjarChambers => DmsgTable(55472),
        Emotes => DmsgTable(55675, 55555),
        EquipmentLocations => DmsgTable(55471),
        EquipmentLocationsAlt => DmsgTable(55666),
        ErrorMessages => DmsgTable(55646, 55526),
        IngameMessages1 => DmsgTable(55648, 55528),
        IngameMessages2 => XiStringTable(55649),
        JobNames => DmsgTable(55467, 55536),
        JobNamesShort => DmsgTable(55468),
        JobPointBonuses => DmsgTable(55694, 55574),
        JobPointGifts => DmsgTable(55674, 55554),
        KeyItems => DmsgTable(55695, 55575),
        MenuItemsDescription => DmsgTable(55651, 55531),
        MenuItemsText => DmsgTable(55652, 55532),
        Merits => DmsgTable(55686, 55566),
        MissionsAcp => DmsgTable(55735, 55615),
        MissionsAmke => DmsgTable(55736, 55616),
        MissionsAsa => DmsgTable(55737, 55617),
        MissionsAssault => DmsgTable(55720, 55600),
        MissionsBastok => DmsgTable(55716, 55596),
        MissionsCampaign => DmsgTable(55724, 55604),
        MissionsCop => DmsgTable(55719, 55599),
        MissionsRov => DmsgTable(55741, 55621),
        MissionsSandoria => DmsgTable(55715, 55595),
        MissionsSoa => DmsgTable(55738, 55618),
        MissionsToau => DmsgTable(55721, 55601),
        MissionsWindurst => DmsgTable(55717, 55597),
        MissionsWotg => DmsgTable(55723, 55603),
        MissionsZilart => DmsgTable(55718, 55598),
        MoblinMazeMongers => DmsgTable(55691, 55571),
        Modifiers => DmsgTable(55689, 55569),
        MonsterFamilies => DmsgTable(55690, 55570),
        MoonPhases => DmsgTable(55660, 55540),
        MountNames => DmsgTable(55681),
        PankrationNames => DmsgTable(55473),
        PolMessages => XiStringTable(55647),
        QuestsAbyssea => DmsgTable(55713, 55593),
        QuestsBastok => DmsgTable(55707, 55587),
        QuestsCoalition => DmsgTable(55740, 55620),
        QuestsJeuno => DmsgTable(55709, 55589),
        QuestsOther => DmsgTable(55710, 55590),
        QuestsOutlands => DmsgTable(55711, 55591),
        QuestsSandoria => DmsgTable(55706, 55586),
        QuestsSoa => DmsgTable(55739, 55619),
        QuestsToau => DmsgTable(55712, 55592),
        QuestsWindurst => DmsgTable(55708, 55588),
        QuestsWotg => DmsgTable(55722, 55602),
        RaceNames => DmsgTable(55469),
        RegionNames => DmsgTable(55654, 55534),
        ServerNames => DmsgTable(55680, 55560),
        SpellNames => DmsgTable(55702, 55582),
        SpellDescriptions => DmsgTable(55734, 55614),
        StatusInfo => StatusInfoTable(87),
        StatusNames => DmsgTable(55725, 55605),
        TimeAndPronouns => XiStringTable(63),
        Titles => DmsgTable(55704, 55584),
        TrustMessages => DmsgTable(55693, 55573),
        Misc1 => DmsgTable(55645, 55525),
        Misc2 => DmsgTable(55653, 55533),
        WeatherTypes => DmsgTable(55657, 55537),

        // Items
        Armor => ItemInfoTable(76, 7),
        Armor2 => ItemInfoTable(55668, 55548),
        Currency => ItemInfoTable(91),
        GeneralItems => ItemInfoTable(73, 4),
        GeneralItems2 => ItemInfoTable(55671, 55551),
        PuppetItems => ItemInfoTable(77, 8),
        UsableItems => ItemInfoTable(74, 5),
        Weapons => ItemInfoTable(75, 6),
        VouchersAndSlips => ItemInfoTable(55667),
        Monipulator => ItemInfoTable(55669),
        Instincts => ItemInfoTable(55670),
        Furniture => FurnitureData(32922),
        AutoTranslate => AutoTranslate(55665),

        // Global dialog
        MonsterSkillNames => Dialog(7035),
        StatusNamesDialog => Dialog(7029),
        EmoteMessages => Dialog(7025),
        SystemMessages1 => Dialog(7023),
        SystemMessages2 => Dialog(7031),
        SystemMessages3 => Dialog(7021),
        SystemMessages4 => Dialog(7027),
        UnityDialogs => Dialog(7039),
    },

    zones: {
        ZoneData => zone_data,
        EntityNames => entities,
        Dialog => dialog,
        Dialog2 => dialog2,
        Events => events,
    }
}
