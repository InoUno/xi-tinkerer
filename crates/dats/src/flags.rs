use crate::serde_bitflags;
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct ValidTargets: u16 {
        // Combined flags
        const Corpse = 0x9D; // CorpseOnly + NPC + Ally + Partymember + Self
        const Object = 0x60;

        // Base flags
        const None = 0x00;
        const SelfTarget = 0x01;
        const Player = 0x02;
        const PartyMember = 0x04;
        const Ally = 0x08;
        const NPC = 0x10;
        const Enemy = 0x20;
        const Unknown = 0x40;
        const CorpseOnly = 0x80;
    }
}
serde_bitflags!(ValidTargets);

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct ItemFlag: u16 {
        // Combined Flags
        const Ex = 0x6040; // NoAuction + NoDelivery + NoTrade

        // Simple Flags - mostly assumed meanings
        const WallHanging = 0x0001; // Used by furnishing like paintings.
        const Flag01 = 0x0002;
        const MysteryBox = 0x0004;  // Can be gained from Gobbie Mystery Box
        const MogGarden = 0x0008;   // Can use in Mog Garden
        const CanSendPOL = 0x0010;
        const Inscribable = 0x0020;
        const NoAuction = 0x0040;
        const Scroll = 0x0080;
        const Linkshell = 0x0100;
        const CanUse = 0x0200;
        const CanTradeNPC = 0x0400;
        const CanEquip = 0x0800;
        const NoSale = 0x1000;
        const NoDelivery = 0x2000;
        const NoTradePC = 0x4000;
        const Rare = 0x8000;
    }
}
serde_bitflags!(ItemFlag);

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct EquipmentSlot: u16 {
        // Combined
        const Ears = 0x1800;
        const Rings = 0x6000;

        // Base
        const None = 0x0000;
        const Main = 0x0001;
        const Sub = 0x0002;
        const Range = 0x0004;
        const Ammo = 0x0008;
        const Head = 0x0010;
        const Body = 0x0020;
        const Hands = 0x0040;
        const Legs = 0x0080;
        const Feet = 0x0100;
        const Neck = 0x0200;
        const Waist = 0x0400;
        const LEar = 0x0800;
        const REar = 0x1000;
        const LRing = 0x2000;
        const RRing = 0x4000;
        const Back = 0x8000;
    }
}
serde_bitflags!(EquipmentSlot);

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct Race: u16 {
        const All = 0x01FE;
        // Gender grouping
        const AnyMale = 0x012A;
        const AnyFemale = 0x00D4;
        // Race grouping
        const Hume = 0x0006;
        const Elvaan = 0x0018;
        const Tarutaru = 0x0060;

        // Base races
        const HumeMale = 0x0002;
        const HumeFemale = 0x0004;
        const ElvaanMale = 0x0008;
        const ElvaanFemale = 0x0010;
        const TarutaruMale = 0x0020;
        const TarutaruFemale = 0x0040;
        const Mithra = 0x0080;
        const Galka = 0x0100;
    }
}
serde_bitflags!(Race);

bitflags! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct JobFlag: u32 {
        const All = 0x007FFFFE;

        const WAR = 0x00000002;
        const MNK = 0x00000004;
        const WHM = 0x00000008;
        const BLM = 0x00000010;
        const RDM = 0x00000020;
        const THF = 0x00000040;
        const PLD = 0x00000080;
        const DRK = 0x00000100;
        const BST = 0x00000200;
        const BRD = 0x00000400;
        const RNG = 0x00000800;
        const SAM = 0x00001000;
        const NIN = 0x00002000;
        const DRG = 0x00004000;
        const SMN = 0x00008000;
        const BLU = 0x00010000;
        const COR = 0x00020000;
        const PUP = 0x00040000;
        const DNC = 0x00080000;
        const SCH = 0x00100000;
        const GEO = 0x00200000;
        const RUN = 0x00400000;
        const MON = 0x00800000;

        const JOB24 = 0x01000000;
        const JOB25 = 0x02000000;
        const JOB26 = 0x04000000;
        const JOB27 = 0x08000000;
        const JOB28 = 0x10000000;
        const JOB29 = 0x20000000;
        const JOB30 = 0x40000000;
        const JOB31 = 0x80000000;
    }
}
serde_bitflags!(JobFlag);
