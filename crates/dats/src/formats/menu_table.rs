use std::collections::BTreeMap;

use crate::{
    enums::{Element, JobEnum, MagicType, SkillType},
    serde_base64, serde_hex,
    utils::{decode_data_block_masked, encode_data_block_masked},
};
use anyhow::{anyhow, Result};
use common::{
    byte_walker::{BufferedByteWalker, ByteWalker},
    expect_msg,
    vec_byte_walker::VecByteWalker,
    writing_byte_walker::WritingByteWalker,
};
use serde_derive::{Deserialize, Serialize};

use crate::{dat_format::DatFormat, enums::AbilityType, flags::ValidTargets};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "entries")]
pub enum Section {
    Mnc2(#[serde(with = "serde_base64")] Vec<u8>),
    Mon_(#[serde(with = "serde_base64")] Vec<u8>),
    Levc(#[serde(with = "serde_base64")] Vec<u8>),
    Comm(Vec<AbilityInfo>),
    Mgc_(Vec<MagicInfo>),

    End,
}

pub trait SectionInfo: Sized {
    fn entry_size() -> usize;
    fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self>;

    fn parse_all<T: ByteWalker>(walker: &mut T, section_size: u32) -> Result<Vec<Self>> {
        if section_size as usize % Self::entry_size() != 0 {
            return Err(anyhow!(
                "Expected byte length to be divisible by {}. Got length {}, which has a remainder of {}.",
                Self::entry_size(),
                section_size,
                section_size as usize % Self::entry_size()
            ));
        }

        let bytes = walker.take_bytes(section_size as usize)?;
        let mut section_walker = BufferedByteWalker::on(bytes);
        let mut entries = Vec::with_capacity(section_size as usize / Self::entry_size());
        while section_walker.remaining() > 0 {
            entries.push(Self::parse(&mut section_walker)?);
        }
        Ok(entries)
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()>;
    fn write_all<T: WritingByteWalker>(entries: &Vec<Self>, walker: &mut T) -> Result<u32> {
        let start_offset = walker.offset();
        for entry in entries {
            entry.write(walker)?;
        }
        let len = walker.offset() - start_offset;
        Ok(len as u32)
    }
}

impl Section {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Section> {
        let section_code = String::from_utf8(walker.take_bytes(4)?.to_vec())?;
        let size_info = walker.step::<u32>()?;
        let section_size = ((size_info & 0xFFFFFF80) >> 3) - 16;
        let unknown_section_info = (size_info & 0x7F) as u8;

        walker.expect_n_msg::<u8>(0, 8, "Padding after section size info")?;

        let section = match section_code.as_str() {
            "mnc2" => Section::Mnc2(walker.take_bytes(section_size as usize)?.to_vec()),
            "mon_" => Section::Mon_(walker.take_bytes(section_size as usize)?.to_vec()),
            "levc" => Section::Levc(walker.take_bytes(section_size as usize)?.to_vec()),
            "comm" => Section::Comm(AbilityInfo::parse_all(walker, section_size)?),
            "mgc_" => Section::Mgc_(MagicInfo::parse_all(walker, section_size)?),
            "end\0" => Section::End,
            _ => {
                return Err(anyhow!("Unknown section code: {}", section_code));
            }
        };

        if unknown_section_info != section.get_unknown_section_info() {
            return Err(anyhow!(
                "Expected unknown section info to be {}, but found {}.",
                section.get_unknown_section_info(),
                unknown_section_info
            ));
        }

        Ok(section)
    }

    fn get_unknown_section_info(&self) -> u8 {
        match self {
            Section::Mnc2(_) => 4,
            Section::Mon_(_) => 4,
            Section::Levc(_) => 4,
            Section::Comm(_) => 83,
            Section::Mgc_(_) => 73,
            Section::End => 0,
        }
    }

    fn get_section_info(&self, content_len: u32) -> u32 {
        ((content_len + 16) << 3) + self.get_unknown_section_info() as u32
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        match self {
            Section::Mnc2(bytes) => {
                walker.write_str("mnc2");
                walker.write(self.get_section_info(bytes.len() as u32));
                walker.skip(8);
                walker.write_bytes(&bytes);
            }
            Section::Mon_(bytes) => {
                walker.write_str("mon_");
                walker.write(self.get_section_info(bytes.len() as u32));
                walker.skip(8);
                walker.write_bytes(&bytes);
            }
            Section::Levc(bytes) => {
                walker.write_str("levc");
                walker.write(self.get_section_info(bytes.len() as u32));
                walker.skip(8);
                walker.write_bytes(&bytes);
            }
            Section::Comm(comm) => {
                walker.write_str("comm");
                let size_info_offset = walker.offset();
                walker.skip(12);
                let content_len = AbilityInfo::write_all(comm, walker)?;
                walker.write_at(size_info_offset, self.get_section_info(content_len));
            }
            Section::Mgc_(magic) => {
                walker.write_str("mgc_");
                let size_info_offset = walker.offset();
                walker.skip(12);
                let content_len = MagicInfo::write_all(magic, walker)?;
                walker.write_at(size_info_offset, self.get_section_info(content_len));
            }

            Section::End => unreachable!(),
        };

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbilityInfo {
    id: u16,
    ability_type: AbilityType,
    icon_id: u8,
    mp_cost: u16,
    unknown1: u16,
    shared_timer_id: u16,
    valid_targets: ValidTargets,
    tp_cost: i16,

    #[serde(with = "serde_hex")]
    unknowns: Vec<u8>,
}

impl SectionInfo for AbilityInfo {
    #[inline]
    fn entry_size() -> usize {
        48
    }

    fn parse<T: ByteWalker>(walker: &mut T) -> Result<AbilityInfo> {
        let mut data_bytes = walker.take_bytes(Self::entry_size())?.to_vec();
        decode_data_block_masked(&mut data_bytes);
        let mut data_walker = BufferedByteWalker::on(data_bytes);

        let info = AbilityInfo {
            id: data_walker.step::<u16>()?,
            ability_type: AbilityType::from(data_walker.step::<u8>()?),
            icon_id: data_walker.step::<u8>()?,
            unknown1: data_walker.step::<u16>()?,
            mp_cost: data_walker.step::<u16>()?,
            shared_timer_id: data_walker.step::<u16>()?,
            valid_targets: ValidTargets::from_bits(data_walker.step::<u16>()?).unwrap_or_default(),
            tp_cost: data_walker.step::<i16>()?,
            unknowns: data_walker
                .take_bytes(data_walker.remaining() - 1)?
                .to_vec(),
        };

        data_walker.expect_msg::<u8>(0xFF, "End of ability marker")?;

        Ok(info)
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        let mut data_walker = VecByteWalker::with_size(Self::entry_size());

        data_walker.write(self.id);
        data_walker.write::<u8>(self.ability_type.into());
        data_walker.write(self.icon_id);
        data_walker.write(self.unknown1);
        data_walker.write(self.mp_cost);
        data_walker.write(self.shared_timer_id);
        data_walker.write(self.valid_targets.bits());
        data_walker.write(self.tp_cost);
        data_walker.write_bytes(&self.unknowns);

        data_walker.write::<u8>(0xFF);

        encode_data_block_masked(data_walker.as_mut_slice());

        walker.write_bytes(data_walker.as_slice());

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MagicInfo {
    index: u16,
    magic_type: MagicType,
    element: Element,
    valid_targets: ValidTargets,
    skill_type: SkillType,
    mp_cost: u16,
    cast_time: u8,
    recast_time: u8,
    level_required: BTreeMap<JobEnum, u16>,
    id: u16,
    icon_id: u8,

    #[serde(with = "serde_hex")]
    unknowns: Vec<u8>,
}

impl SectionInfo for MagicInfo {
    #[inline]
    fn entry_size() -> usize {
        100
    }

    fn parse<T: ByteWalker>(walker: &mut T) -> Result<MagicInfo> {
        let mut data_bytes = walker.take_bytes(Self::entry_size())?.to_vec();
        decode_data_block_masked(&mut data_bytes);
        let mut data_walker = BufferedByteWalker::on(data_bytes);

        let info = MagicInfo {
            index: data_walker.step::<u16>()?,
            magic_type: MagicType::from(data_walker.step::<u16>()?),
            element: Element::try_from(data_walker.step::<u16>()?)?,
            valid_targets: ValidTargets::from_bits(data_walker.step::<u16>()?).unwrap_or_default(),
            skill_type: SkillType::from(data_walker.step::<u16>()? as u8),
            mp_cost: data_walker.step()?,
            cast_time: data_walker.step()?,
            recast_time: data_walker.step()?,
            level_required: (0..24)
                .into_iter()
                .filter_map(|idx| {
                    let level = data_walker.step::<i16>().ok()?;
                    if level != -1 {
                        Some((JobEnum::from(idx), level as u16))
                    } else {
                        None
                    }
                })
                .collect(),
            id: data_walker.step()?,
            icon_id: data_walker.step()?,

            unknowns: data_walker
                .take_bytes(data_walker.remaining() - 1)?
                .to_vec(),
        };

        data_walker.expect_msg::<u8>(0xFF, "End of magic marker")?;

        Ok(info)
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        let mut data_walker = VecByteWalker::with_size(Self::entry_size());

        data_walker.write(self.index);
        data_walker.write::<u16>(self.magic_type.into());
        data_walker.write::<u16>(self.element.into());
        data_walker.write::<u16>(self.valid_targets.bits());

        let skill_type: u8 = self.skill_type.into();
        data_walker.write::<u16>(skill_type as u16);
        data_walker.write(self.mp_cost);
        data_walker.write(self.cast_time);
        data_walker.write(self.recast_time);

        for job_idx in 0..24 {
            let job = JobEnum::from(job_idx);
            let level_required = self
                .level_required
                .get(&job)
                .copied()
                .map(|level| level as i16)
                .unwrap_or(-1);

            data_walker.write(level_required);
        }

        data_walker.write(self.id);
        data_walker.write(self.icon_id);
        data_walker.write_bytes(&self.unknowns);

        data_walker.write::<u8>(0xFF);

        encode_data_block_masked(data_walker.as_mut_slice());

        walker.write_bytes(data_walker.as_slice());

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonInfo {
    #[serde(with = "serde_hex")]
    unknowns: Vec<u8>,
}

impl SectionInfo for MonInfo {
    #[inline]
    fn entry_size() -> usize {
        64
    }

    fn parse<T: ByteWalker>(walker: &mut T) -> Result<MonInfo> {
        let data_bytes = walker.take_bytes(Self::entry_size())?.to_vec();
        let mut data_walker = BufferedByteWalker::on(data_bytes);

        let info = MonInfo {
            unknowns: data_walker.take_bytes(data_walker.remaining())?.to_vec(),
        };

        Ok(info)
    }

    fn write<T: WritingByteWalker>(&self, _walker: &mut T) -> Result<()> {
        todo!()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MenuTable {
    sections: Vec<Section>,
}

impl MenuTable {
    pub fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        walker.expect_utf8_str("menu")?;
        walker.expect::<u32>(0x101)?;
        walker.expect_n_msg::<u8>(0, 24, "Padding after menu tag")?;

        let mut sections = vec![];
        loop {
            let section = Section::parse(walker)?;
            if matches!(section, Section::End) {
                break;
            }
            sections.push(section);
        }

        expect_msg(0, walker.remaining(), "End of sections")?;

        Ok(MenuTable { sections })
    }

    pub fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        walker.write_str("menu");
        walker.write::<u32>(0x101);
        walker.write_bytes(&vec![0; 24]);

        for section in &self.sections {
            section.write(walker)?;
        }

        walker.write_str("end\0");
        walker.write::<u32>(16 << 3);
        walker.write_bytes(&vec![0; 8]);

        Ok(())
    }
}

impl DatFormat for MenuTable {
    fn from<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        MenuTable::parse(walker)
    }

    fn check_type<T: ByteWalker>(walker: &mut T) -> Result<()> {
        walker.expect_utf8_str("menu")?;

        Ok(())
    }

    fn write<T: WritingByteWalker>(&self, walker: &mut T) -> Result<()> {
        self.write(walker)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufWriter, path::PathBuf};

    use crate::dat_format::DatFormat;

    use super::MenuTable;

    #[test]
    pub fn menu_table() {
        let mut dat_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dat_path.push("resources/test/menu.DAT");

        MenuTable::check_path(&dat_path).unwrap();
        let res = MenuTable::from_path_checked(&dat_path).unwrap();

        let file = File::create("menu.yml").unwrap();
        serde_yaml::to_writer(BufWriter::new(file), &res).unwrap();
    }
}
