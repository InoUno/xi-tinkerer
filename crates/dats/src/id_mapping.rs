use std::sync::OnceLock;

use crate::{
    base::{Dat, DatByZone},
    formats::{
        dialog::Dialog, dmsg2_string_table::Dmsg2StringTable, dmsg3_string_table::Dmsg3StringTable,
        entity_names::EntityNames, item_info::ItemInfoTable, status_info::StatusInfoTable,
        xistring_table::XiStringTable,
    },
};

#[derive(Debug)]
pub struct DatIdMapping {
    pub entities: DatByZone<EntityNames>,
    pub dialog: DatByZone<Dialog>,
    pub dialog2: DatByZone<Dialog>,

    // Global dialog
    pub monster_skill_names: Dat<Dialog>,
    pub status_names_dialog: Dat<Dialog>,
    pub emote_messages: Dat<Dialog>,
    pub system_messages_1: Dat<Dialog>,
    pub system_messages_2: Dat<Dialog>,
    pub system_messages_3: Dat<Dialog>,
    pub system_messages_4: Dat<Dialog>,
    pub unity_dialogs: Dat<Dialog>,

    // String tables
    pub ability_names: Dat<Dmsg3StringTable>,
    pub ability_descriptions: Dat<Dmsg3StringTable>,
    pub area_names: Dat<Dmsg2StringTable>,
    pub area_names_alt: Dat<Dmsg2StringTable>,
    pub character_select: Dat<Dmsg2StringTable>,
    pub chat_filter_types: Dat<Dmsg2StringTable>,
    pub day_names: Dat<Dmsg2StringTable>,
    pub directions: Dat<Dmsg2StringTable>,
    pub equipment_locations: Dat<Dmsg2StringTable>,
    pub error_messages: Dat<Dmsg2StringTable>,
    pub ingame_messages_1: Dat<Dmsg2StringTable>,
    pub ingame_messages_2: Dat<XiStringTable>,
    pub job_names: Dat<Dmsg2StringTable>,
    pub key_items: Dat<Dmsg3StringTable>,
    pub menu_items_description: Dat<Dmsg2StringTable>,
    pub menu_items_text: Dat<Dmsg2StringTable>,

    pub moon_phases: Dat<Dmsg2StringTable>,
    pub pol_messages: Dat<XiStringTable>,
    pub race_names: Dat<Dmsg2StringTable>,
    pub region_names: Dat<Dmsg2StringTable>,
    pub spell_names: Dat<Dmsg3StringTable>,
    pub spell_descriptions: Dat<Dmsg3StringTable>,
    pub status_info: Dat<StatusInfoTable>,
    pub status_names: Dat<Dmsg2StringTable>,
    pub time_and_pronouns: Dat<XiStringTable>,
    pub titles: Dat<Dmsg3StringTable>,
    pub misc1: Dat<Dmsg2StringTable>,
    pub misc2: Dat<Dmsg2StringTable>,
    pub weather_types: Dat<Dmsg2StringTable>,

    // Item data
    pub armor: Dat<ItemInfoTable>,
    pub armor2: Dat<ItemInfoTable>,
    pub currency: Dat<ItemInfoTable>,
    pub general_items: Dat<ItemInfoTable>,
    pub general_items2: Dat<ItemInfoTable>,
    pub puppet_items: Dat<ItemInfoTable>,
    pub usable_items: Dat<ItemInfoTable>,
    pub weapons: Dat<ItemInfoTable>,
    pub vouchers_and_slips: Dat<ItemInfoTable>,
    pub monipulator: Dat<ItemInfoTable>,
    pub instincts: Dat<ItemInfoTable>,
}

static DAT_ID_MAPPING: OnceLock<DatIdMapping> = OnceLock::new();

impl DatIdMapping {
    pub fn get() -> &'static Self {
        DAT_ID_MAPPING.get_or_init(|| {
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

            // Secondary dialog text
            let mut dialog2 = DatByZone::default();
            // Just whitegate?
            dialog2.insert(50, 57945);

            Self {
                entities,
                dialog,
                dialog2,

                // Global dialog
                monster_skill_names: 07035.into(),
                status_names_dialog: 07029.into(),
                emote_messages: 07025.into(),
                system_messages_1: 07023.into(),
                system_messages_2: 07031.into(),
                system_messages_3: 07021.into(),
                system_messages_4: 07027.into(),
                unity_dialogs: 07039.into(),

                // String tables
                ability_names: 55701.into(),
                ability_descriptions: 55733.into(),
                area_names: 55465.into(),
                area_names_alt: 55661.into(),
                character_select: 55470.into(),
                chat_filter_types: 55650.into(),
                day_names: 55658.into(),
                directions: 55659.into(),
                equipment_locations: 55471.into(),
                error_messages: 55646.into(),
                ingame_messages_1: 55648.into(),
                ingame_messages_2: 55649.into(),
                job_names: 55467.into(),
                key_items: 55695.into(),
                menu_items_description: 55651.into(),
                menu_items_text: 55652.into(),
                moon_phases: 55660.into(),
                pol_messages: 55647.into(),
                race_names: 55469.into(),
                region_names: 55654.into(),
                spell_names: 55702.into(),
                spell_descriptions: 55734.into(),
                status_info: 00087.into(),
                status_names: 55725.into(),
                time_and_pronouns: 00063.into(),
                titles: 55704.into(),
                misc1: 55645.into(),
                misc2: 55653.into(),
                weather_types: 55657.into(),

                // Item data
                armor: 00076.into(),
                armor2: 55668.into(),
                currency: 00091.into(),
                general_items: 00073.into(),
                general_items2: 55671.into(),
                puppet_items: 00077.into(),
                usable_items: 00074.into(),
                weapons: 00075.into(),
                vouchers_and_slips: 55667.into(),
                monipulator: 55669.into(),
                instincts: 55670.into(),
            }
        })
    }
}
