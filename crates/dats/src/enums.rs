// Need this until rust-analyzer has fix for the issue: https://github.com/rust-lang/rust-analyzer/issues/15344
#![allow(non_upper_case_globals)]

use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SkillType {
    #[default]
    None = 0x00,

    HandToHand = 0x01,
    Dagger = 0x02,
    Sword = 0x03,
    GreatSword = 0x04,
    Axe = 0x05,
    GreatAxe = 0x06,
    Scythe = 0x07,
    PoleArm = 0x08,
    Katana = 0x09,
    GreatKatana = 0x0a,
    Club = 0x0b,
    Staff = 0x0c,
    AutomatonMelee = 0x16,
    AutomatonRange = 0x17,
    AutomatonMagic = 0x18,
    Ranged = 0x19,
    Marksmanship = 0x1a,
    Thrown = 0x1b,
    DivineMagic = 0x20,
    HealingMagic = 0x21,
    EnhancingMagic = 0x22,
    EnfeeblingMagic = 0x23,
    ElementalMagic = 0x24,
    DarkMagic = 0x25,
    SummoningMagic = 0x26,
    Ninjutsu = 0x27,
    Singing = 0x28,
    StringInstrument = 0x29,
    WindInstrument = 0x2a,
    BlueMagic = 0x2b,
    Geomancy = 0x2c,
    Handbell = 0x2d,
    Fishing = 0x30,

    Woodworking = 0x31,
    Smithing = 0x32,
    Goldsmithing = 0x33,
    Clothcraft = 0x34,
    Leathercraft = 0x35,
    Bonecraft = 0x36,
    Alchemy = 0x37,
    Cooking = 0x38,

    Special = 0xff,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ItemType {
    None = 0,

    Item = 1,
    QuestItem = 2,
    Fish = 3,
    Weapon = 4,
    Armor = 5,
    Linkshell = 6,
    UsableItem = 7,
    Crystal = 8,
    Currency = 9,
    Furnishing = 10,
    Plant = 11,
    Flowerpot = 12,
    PuppetItem = 13,
    Mannequin = 14,
    Book = 15,
    RacingForm = 16,
    BettingSlip = 17,
    SoulPlate = 18,
    Reflector = 19,

    LotteryTicket = 21,
    MazeTabulaM = 22,
    MazeTabulaR = 23,
    MazeVoucher = 24,
    MazeRune = 25,

    StorageSlip = 27,

    Instinct = 30,

    #[num_enum(catch_all)]
    #[serde(untagged)]
    Unknown(u16),
}

impl Default for ItemType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum PuppetSlot {
    #[default]
    None = 0,

    Head = 1,
    Body = 2,
    Attachment = 3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum Element {
    Fire = 0x00,
    Ice = 0x01,
    Air = 0x02,
    Earth = 0x03,
    Thunder = 0x04,
    Water = 0x05,
    Light = 0x06,
    Dark = 0x07,
    Special = 0x0F,
    Undecided = 0xFFFF,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AbilityType {
    General = 0,
    Job = 1,
    Pet = 2,
    Weapon = 3,
    Trait = 4,
    Unknown5 = 5,
    BloodPactRage = 6,
    Unknown7 = 7,
    Corsair = 8,
    CorsairShot = 9,
    BloodPactWard = 10,
    Samba = 11,
    Waltz = 12,
    Step = 13,
    Flourish1 = 14,
    Scholar = 15,
    Jig = 16,
    Flourish2 = 17,
    Monster = 18,
    Flourish3 = 19,
    Weaponskill = 20,
    Rune = 21,
    Ward = 22,
    Effusion = 23,

    #[num_enum(catch_all)]
    #[serde(untagged)]
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum MagicType {
    None = 0,
    WhiteMagic = 1,
    BlackMagic = 2,
    SummonerPact = 3,
    Ninjutsu = 4,
    BardSong = 5,
    BlueMagic = 6,
    Geomancy = 7,

    #[num_enum(catch_all)]
    #[serde(untagged)]
    Unknown(u16),
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
pub enum JobEnum {
    WAR = 1,
    MNK = 2,
    WHM = 3,
    BLM = 4,
    RDM = 5,
    THF = 6,
    PLD = 7,
    DRK = 8,
    BST = 9,
    BRD = 10,
    RNG = 11,
    SAM = 12,
    NIN = 13,
    DRG = 14,
    SMN = 15,
    BLU = 16,
    COR = 17,
    PUP = 18,
    DNC = 19,
    SCH = 20,
    GEO = 21,
    RUN = 22,
    MON = 23,

    #[num_enum(catch_all)]
    #[serde(untagged)]
    Unknown(u8),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TryFromPrimitive, IntoPrimitive,
)]
#[repr(u32)]
pub enum EnglishArticle {
    A = 0,
    An = 1,
    PairOf = 2,
    SuitsOf = 3,
}
