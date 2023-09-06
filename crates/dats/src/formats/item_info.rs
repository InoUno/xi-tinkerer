use anyhow::{anyhow, Ok, Result};
use common::{
    byte_walker::{BufferedByteWalker, ByteWalker},
    get_padding,
    vec_byte_walker::VecByteWalker,
    writing_byte_walker::WritingByteWalker,
};
use encoding::{decoder::Decoder, encoder::Encoder};
use serde_derive::{Deserialize, Serialize};

use crate::{
    dat_format::DatFormat,
    enums::{Element, EnglishArticle, ItemType, PuppetSlot, SkillType},
    flags::{EquipmentSlot, ItemFlag, JobFlag, Race, ValidTargets},
    serde_base64,
    utils::{get_nibble, rotate_all},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ItemInfo {
    id: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    strings: Option<ItemStrings>,

    flags: ItemFlag,
    stack_size: u16,
    item_type: ItemType,
    resource_id: u16,
    valid_targets: ValidTargets,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    equipment: Option<EquipmentData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    weapon: Option<WeaponData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    puppet: Option<PuppetItemData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    instinct: Option<InstinctData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    furnishing: Option<FurnishingData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    usable_item: Option<UsableItemData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    currency: Option<CurrencyData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    slip: Option<SlipData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    monipulator: Option<MonipulatorData>,

    #[serde(with = "serde_base64")]
    icon_bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItemStrings {
    #[serde(untagged)]
    Name { name: String },

    #[serde(untagged)]
    English {
        name: String,
        article_type: EnglishArticle,
        singular_name: String,
        plural_name: String,
        description: String,
    },
}

#[derive(Debug, Clone)]
pub enum ItemStringContent {
    Number(u32),
    StringBytes(Vec<u8>),
}

impl ItemStringContent {
    pub fn from_string(str: &String) -> Result<Self> {
        let mut string_walker = VecByteWalker::with_size(28);

        // Start of string and initial padding
        string_walker.write::<u32>(1);
        for _ in 0..6 {
            string_walker.write::<u32>(0);
        }

        string_walker.write_bytes(&Encoder::encode_simple(str)?);
        string_walker.write::<u8>(0); // End of string

        // Alignment padding
        let padding = get_padding(string_walker.offset());
        for _ in 0..padding {
            string_walker.write::<u8>(0);
        }
        Ok(ItemStringContent::StringBytes(string_walker.into_vec()))
    }

    pub fn from_article(article: impl Into<u32>) -> Self {
        ItemStringContent::Number(article.into())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemCategory {
    Unknown,
    Currency,
    Item,
    Armor,
    Weapon,
    PuppetItem,
    UsableItem,
    Slip,
    Instinct,
    Monipulator,
}

impl ItemCategory {
    pub fn from_id(id: u32) -> Self {
        match id {
            0xFFFF => ItemCategory::Currency,
            0..=0xFFF => ItemCategory::Item,
            0x1000..=0x1FFF => ItemCategory::UsableItem,
            0x2000..=0x21FF => ItemCategory::PuppetItem,
            0x2200..=0x27FF => ItemCategory::Item,
            0x2800..=0x3FFF => ItemCategory::Armor,
            0x4000..=0x59FF => ItemCategory::Weapon,
            0x5A00..=0x6FFF => ItemCategory::Armor,
            0x7000..=0x73FF => ItemCategory::Slip,
            0x7400..=0x77FF => ItemCategory::Instinct,
            0x7800..=0xF1FF => ItemCategory::Monipulator,
            0xF200.. => ItemCategory::Item,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EquipmentData {
    level: u16,
    slots: EquipmentSlot,
    races: Race,
    jobs: JobFlag,
    superior_level: u16,
    shield_size: u16,

    max_charges: u8,
    casting_time: u8,
    use_delay: u16,
    reuse_delay: u32,
    unknown1: u16,
    ilevel: u8,
    unknown2: u8,
    unknown3: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeaponData {
    damage: u16,
    delay: u16,
    dps: u16,
    skill_type: SkillType,
    jug_size: u8,
    unknown1: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PuppetItemData {
    slot: PuppetSlot,
    element_charge: ElementValues,
    unknown1: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementValues {
    fire: u8,
    ice: u8,
    wind: u8,
    earth: u8,
    lightning: u8,
    water: u8,
    light: u8,
    dark: u8,
}

impl From<u32> for ElementValues {
    fn from(value: u32) -> Self {
        ElementValues {
            fire: get_nibble(value, 0),
            ice: get_nibble(value, 1),
            wind: get_nibble(value, 2),
            earth: get_nibble(value, 3),
            lightning: get_nibble(value, 4),
            water: get_nibble(value, 5),
            light: get_nibble(value, 6),
            dark: get_nibble(value, 7),
        }
    }
}

impl From<ElementValues> for u32 {
    fn from(value: ElementValues) -> Self {
        value.fire as u32
            + ((value.ice as u32) << (4 * 1))
            + ((value.wind as u32) << (4 * 2))
            + ((value.earth as u32) << (4 * 3))
            + ((value.lightning as u32) << (4 * 4))
            + ((value.water as u32) << (4 * 5))
            + ((value.light as u32) << (4 * 6))
            + ((value.dark as u32) << (4 * 7))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstinctData {
    unknown1: u32,
    unknown2: u32,
    unknown3: u16,
    instinct_cost: u16,
    unknown4: u16,
    unknown5: u32,
    unknown6: u32,
    unknown7: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FurnishingData {
    element: Element,
    storage_slots: u32,
    unknown3: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsableItemData {
    activation_time: u16,
    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyData {
    unknown1: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlipData {
    unknown1: u16,
    unknowns: [u32; 17],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonipulatorData {
    unknown1: u16,
    unknowns: [u32; 24],
}

impl ItemInfo {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<ItemInfo> {
        let mut item_bytes = walker.take_bytes(0xC00)?.to_vec();
        rotate_all(&mut item_bytes, 5);

        // Parse the icon
        let mut icon_walker = BufferedByteWalker::on(&item_bytes[0x280..]);
        let icon_size = icon_walker.step::<u32>()?;
        let icon_bytes = icon_walker.take_bytes(icon_size as usize)?.to_vec();

        icon_walker.expect_n_msg::<u8>(0, icon_walker.remaining() - 1, "Padding after icon")?;
        icon_walker.expect_msg::<u8>(0xFF, "End of icon bytes")?;

        // Parse the data
        let mut data_walker: BufferedByteWalker<&[u8]> =
            BufferedByteWalker::on(&item_bytes[..0x280]);

        let mut item_info = ItemInfo {
            icon_bytes,
            ..Default::default()
        };

        item_info.id = data_walker.step::<u32>()?;
        let item_category = ItemCategory::from_id(item_info.id);

        // TODO: Monipulators seems to have a totally different structure than other items,
        //       since the values it gets for the following are non-sensical.

        item_info.flags = ItemFlag::from_bits(data_walker.step::<u16>()?).unwrap_or_default();
        item_info.stack_size = data_walker.step::<u16>()?;
        item_info.item_type = ItemType::from(data_walker.step::<u16>()?);
        item_info.resource_id = data_walker.step::<u16>()?;
        item_info.valid_targets =
            ValidTargets::from_bits(data_walker.step::<u16>()?).unwrap_or_default();

        if item_category == ItemCategory::Armor || item_category == ItemCategory::Weapon {
            let level = data_walker.step::<u16>()?;
            let slots = EquipmentSlot::from_bits(data_walker.step::<u16>()?).unwrap_or_default();
            let races = Race::from_bits(data_walker.step::<u16>()?).unwrap_or_default();
            let jobs = JobFlag::from_bits(data_walker.step::<u32>()?).unwrap_or_default();
            let superior_level = data_walker.step::<u16>()?;
            let shield_size = data_walker.step::<u16>()?;

            if item_category == ItemCategory::Weapon {
                item_info.weapon = Some(WeaponData {
                    damage: data_walker.step::<u16>()?,
                    delay: data_walker.step::<u16>()?,
                    dps: data_walker.step::<u16>()?,
                    skill_type: SkillType::try_from(data_walker.step::<u8>()?)?,
                    jug_size: data_walker.step::<u8>()?,
                    unknown1: data_walker.step::<u32>()?,
                });
            }

            let max_charges = data_walker.step::<u8>()?;
            let casting_time = data_walker.step::<u8>()?;
            let use_delay = data_walker.step::<u16>()?;
            let reuse_delay = data_walker.step::<u32>()?;
            let unknown1 = data_walker.step::<u16>()?;
            let ilevel = data_walker.step::<u8>()?;
            let unknown2 = data_walker.step::<u8>()?;
            let unknown3 = data_walker.step::<u32>()?;

            item_info.equipment = Some(EquipmentData {
                level,
                slots,
                races,
                jobs,
                superior_level,
                shield_size,
                max_charges,
                casting_time,
                use_delay,
                reuse_delay,
                unknown1,
                ilevel,
                unknown2,
                unknown3,
            });
        } else if item_category == ItemCategory::PuppetItem {
            item_info.puppet = Some(PuppetItemData {
                slot: PuppetSlot::try_from(data_walker.step::<u16>()?)?,
                element_charge: ElementValues::from(data_walker.step::<u32>()?),
                unknown1: data_walker.step::<u32>()?,
            });
        } else if item_category == ItemCategory::Instinct {
            item_info.instinct = Some(InstinctData {
                unknown1: data_walker.step::<u32>()?,
                unknown2: data_walker.step::<u32>()?,
                unknown3: data_walker.step::<u16>()?,
                instinct_cost: data_walker.step::<u16>()?,
                unknown4: data_walker.step::<u16>()?,
                unknown5: data_walker.step::<u32>()?,
                unknown6: data_walker.step::<u32>()?,
                unknown7: data_walker.step::<u32>()?,
            });
        } else if item_category == ItemCategory::Item {
            item_info.furnishing = Some(FurnishingData {
                element: Element::try_from(data_walker.step::<u16>()?)?,
                storage_slots: data_walker.step::<u32>()?,
                unknown3: data_walker.step::<u32>()?,
            });
        } else if item_category == ItemCategory::UsableItem {
            item_info.usable_item = Some(UsableItemData {
                activation_time: data_walker.step::<u16>()?,
                unknown1: data_walker.step::<u32>()?,
                unknown2: data_walker.step::<u32>()?,
                unknown3: data_walker.step::<u32>()?,
            });
        } else if item_category == ItemCategory::Currency {
            item_info.currency = Some(CurrencyData {
                unknown1: data_walker.step::<u16>()?,
            });
        } else if item_category == ItemCategory::Slip {
            item_info.slip = Some(SlipData {
                unknown1: data_walker.step::<u16>()?,
                unknowns: core::array::from_fn(|_| data_walker.step::<u32>().unwrap_or_default()),
            });
        } else if item_category == ItemCategory::Monipulator {
            item_info.monipulator = Some(MonipulatorData {
                unknown1: data_walker.step::<u16>()?,
                unknowns: core::array::from_fn(|_| data_walker.step::<u32>().unwrap_or_default()),
            });
        }

        // Parse string data
        let content_count = data_walker.step::<u32>()?;
        if content_count > 9 {
            return Err(anyhow!(
                "Unsupported strings content of length: {}",
                content_count
            ));
        }

        let mut metas = Vec::with_capacity(content_count as usize);
        for _ in 0..content_count {
            metas.push((data_walker.step::<u32>()?, data_walker.step::<u32>()?));
        }

        match content_count {
            1 => {
                // Just one string name
                item_info.strings = Some(ItemStrings::Name {
                    name: Self::read_string(&mut data_walker)?,
                });
            }
            5 => {
                // English
                item_info.strings = Some(ItemStrings::English {
                    name: Self::read_string(&mut data_walker)?,
                    article_type: EnglishArticle::try_from(data_walker.step::<u32>()?)?,
                    singular_name: Self::read_string(&mut data_walker)?,
                    plural_name: Self::read_string(&mut data_walker)?,
                    description: Self::read_string(&mut data_walker)?,
                });
            }
            count => {
                return Err(anyhow!("Unsupported string count: {}", count));
            }
        }

        data_walker.expect_n_msg::<u32>(
            0,
            data_walker.remaining() / 4,
            "Zero padding at end of data",
        )?;

        Ok(item_info)
    }

    fn read_string<T: ByteWalker>(walker: &mut T) -> Result<String> {
        walker.expect_msg::<u32>(1, "Expected 1 at start of string.")?;
        walker.expect_n_msg::<u32>(0, 6, "Expected 0 padding before string.")?;

        let text_bytes = walker.step_until(0)?;
        let string = Decoder::decode_simple(text_bytes);

        let alignment_padding = get_padding(text_bytes.len() + 1);
        walker.expect_msg::<u8>(0, "End of string")?;
        walker.expect_n_msg::<u8>(0, alignment_padding, "Expected 0 padding after string.")?;

        string
    }

    pub fn write<T: WritingByteWalker>(&self, outer_walker: &mut T) -> Result<()> {
        let mut walker = VecByteWalker::with_size(0xC00);

        walker.write(self.id);
        walker.write(self.flags.bits());

        // Write item data
        walker.write(self.stack_size);
        walker.write::<u16>(self.item_type.into());
        walker.write(self.resource_id);
        walker.write(self.valid_targets.bits());

        if let Some(equipment) = &self.equipment {
            walker.write(equipment.level);
            walker.write(equipment.slots.bits());
            walker.write(equipment.races.bits());
            walker.write(equipment.jobs.bits());
            walker.write(equipment.superior_level);
            walker.write(equipment.shield_size);

            if let Some(weapon) = &self.weapon {
                walker.write(weapon.damage);
                walker.write(weapon.delay);
                walker.write(weapon.dps);
                walker.write::<u8>(weapon.skill_type.into());
                walker.write(weapon.jug_size);
                walker.write(weapon.unknown1);
            }

            walker.write(equipment.max_charges);
            walker.write(equipment.casting_time);
            walker.write(equipment.use_delay);
            walker.write(equipment.reuse_delay);
            walker.write(equipment.unknown1);
            walker.write(equipment.ilevel);
            walker.write(equipment.unknown2);
            walker.write(equipment.unknown3);
        } else if let Some(puppet) = &self.puppet {
            walker.write::<u16>(puppet.slot.into());
            walker.write::<u32>(puppet.element_charge.into());
            walker.write(puppet.unknown1);
        } else if let Some(instinct) = &self.instinct {
            walker.write(instinct.unknown1);
            walker.write(instinct.unknown2);
            walker.write(instinct.unknown3);
            walker.write(instinct.instinct_cost);
            walker.write(instinct.unknown4);
            walker.write(instinct.unknown5);
            walker.write(instinct.unknown6);
            walker.write(instinct.unknown7);
        } else if let Some(furnishing) = &self.furnishing {
            walker.write::<u16>(furnishing.element.into());
            walker.write(furnishing.storage_slots);
            walker.write(furnishing.unknown3);
        } else if let Some(usable_item) = &self.usable_item {
            walker.write(usable_item.activation_time);
            walker.write(usable_item.unknown1);
            walker.write(usable_item.unknown2);
            walker.write(usable_item.unknown3);
        } else if let Some(currency) = &self.currency {
            walker.write(currency.unknown1);
        } else if let Some(slip) = &self.slip {
            walker.write(slip.unknown1);
            for unknown in slip.unknowns {
                walker.write(unknown);
            }
        } else if let Some(monipulator) = &self.monipulator {
            walker.write(monipulator.unknown1);
            for unknown in monipulator.unknowns {
                walker.write(unknown);
            }
        }

        // Write strings
        let mut string_content = vec![];

        match &self.strings {
            Some(ItemStrings::Name { name }) => {
                string_content.push(ItemStringContent::from_string(name)?);
            }
            Some(ItemStrings::English {
                name,
                article_type,
                singular_name,
                plural_name,
                description,
            }) => {
                string_content.push(ItemStringContent::from_string(name)?);
                string_content.push(ItemStringContent::from_article(*article_type));
                string_content.push(ItemStringContent::from_string(singular_name)?);
                string_content.push(ItemStringContent::from_string(plural_name)?);
                string_content.push(ItemStringContent::from_string(description)?);
            }
            None => {}
        }

        // Write metas
        walker.write::<u32>(string_content.len() as u32);

        let mut current_offset: u32 = string_content.len() as u32 * 8 + 4;
        for content in &string_content {
            match content {
                ItemStringContent::Number(_) => {
                    walker.write::<u32>(current_offset);
                    walker.write::<u32>(1);

                    current_offset += 4;
                }
                ItemStringContent::StringBytes(string_bytes) => {
                    walker.write::<u32>(current_offset);
                    walker.write::<u32>(0);

                    current_offset += string_bytes.len() as u32;
                }
            }
        }

        // Write string content
        for content in &string_content {
            match content {
                ItemStringContent::Number(number) => {
                    walker.write(*number);
                }
                ItemStringContent::StringBytes(string_bytes) => {
                    walker.write_bytes(string_bytes);
                }
            }
        }

        // Write icon bytes
        walker.goto(0x280);
        walker.write(self.icon_bytes.len() as u32);
        walker.write_bytes(&self.icon_bytes);
        walker.write_at::<u8>(0xC00 - 1, 0xFF);

        rotate_all(walker.as_mut_slice(), 3);
        outer_walker.write_bytes(walker.as_slice());

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ItemInfoTable {
    items: Vec<ItemInfo>,
}

const ENTRY_SIZE: usize = 0xC00;

impl ItemInfoTable {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        if walker.len() % ENTRY_SIZE != 0 {
            return Err(anyhow!("Length does not match a item info DAT."));
        }

        let entry_count = walker.len() / ENTRY_SIZE;
        let mut items = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            items.push(ItemInfo::parse(walker)?);
        }

        Ok(ItemInfoTable { items })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.set_size(self.items.len() * ENTRY_SIZE);

        for item in &self.items {
            item.write(walker)?;
        }

        Ok(())
    }
}

impl DatFormat for ItemInfoTable {
    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        ItemInfoTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        if walker.len() % ENTRY_SIZE != 0 {
            return Err(anyhow!("Length does not match a item info DAT."));
        }

        // Parse one item info to check.
        ItemInfo::parse(walker)?;

        Ok(())
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{dat_format::DatFormat, enums::EnglishArticle};

    use super::{ItemInfoTable, ItemStrings};

    #[test]
    pub fn weapons() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/weapons.DAT");

        ItemInfoTable::check_path(&dat_path).unwrap();
        let res = ItemInfoTable::from_path_checked(&dat_path).unwrap();

        if let ItemStrings::English {
            name,
            article_type,
            singular_name,
            plural_name,
            description,
        } = res.items[4329].strings.as_ref().unwrap()
        {
            assert_eq!(name, "Excalipoor");
            assert_eq!(article_type, &EnglishArticle::An);
            assert_eq!(singular_name, "Excalipoor");
            assert_eq!(plural_name, "Excalipoors");
            assert_eq!(description, "DMG:1 Delay:240");
        } else {
            panic!("Expected english strings")
        }
    }

    #[test]
    pub fn armor2() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/armor2.DAT");

        ItemInfoTable::check_path(&dat_path).unwrap();
        let res = ItemInfoTable::from_path_checked_during(&dat_path).unwrap();

        if let ItemStrings::English {
            name,
            article_type,
            singular_name,
            plural_name,
            description,
        } = res.items[3827].strings.as_ref().unwrap()
        {
            assert_eq!(name, "Voodoo Mail");
            assert_eq!(article_type, &EnglishArticle::SuitsOf);
            assert_eq!(singular_name, "voodoo mail");
            assert_eq!(plural_name, "suits of voodoo mail");
            assert_eq!(description, "The envious aura that looms over\nthis mail seems to invite utter\nruin to descend upon its bearer.");
        } else {
            panic!("Expected english strings")
        }
    }
}
