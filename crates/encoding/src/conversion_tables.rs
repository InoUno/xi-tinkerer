use std::{
    collections::{hash_map::Entry, HashMap},
    sync::OnceLock,
};

pub struct ConversionTable;

static REVERSE_TABLE: OnceLock<HashMap<u16, u16>> = OnceLock::new();

const EMPTY_TABLE: [u8; 512] = [0xFF; 512];

impl ConversionTable {
    pub fn lookup(table: u8, idx: u8) -> u16 {
        let lookup_idx = idx as usize * 2;

        u16::from_le_bytes(
            Self::get_table(table)[lookup_idx..lookup_idx + 2]
                .try_into()
                .unwrap(),
        )
    }

    pub fn rev_lookup(input: u16) -> u16 {
        REVERSE_TABLE
            .get_or_init(|| {
                let mut map = HashMap::new();

                let index_table = Self::get_table(0x00);

                for first_byte in 0x00u8..=0xFF {
                    let first_idx = first_byte as usize * 2;
                    let first_short = u16::from_le_bytes(
                        index_table[first_idx..first_idx + 2].try_into().unwrap(),
                    );

                    // Check if it needs a seondary lookup
                    if first_short == 0xFFFE {
                        let second_table = Self::get_table(first_byte);

                        if second_table as *const [u8] == &EMPTY_TABLE as *const [u8] {
                            // It's the empty table.
                            continue;
                        }

                        for second_byte in 0x00u8..=0xFF {
                            let second_idx = second_byte as usize * 2;
                            let second_short = u16::from_le_bytes(
                                second_table[second_idx..second_idx + 2].try_into().unwrap(),
                            );

                            if second_short == 0xFFFF {
                                // No conversion
                                continue;
                            }

                            let value = u16::from_le_bytes([first_byte, second_byte]);
                            match map.entry(second_short) {
                                Entry::Occupied(mut occupied) => {
                                    // Duplicate character conversion
                                    occupied.insert(value);
                                }
                                Entry::Vacant(vacant) => {
                                    vacant.insert(value);
                                }
                            }
                        }
                    } else if first_short == 0xFFFF {
                        // No conversion
                        continue;
                    } else {
                        let value = u16::from_le_bytes([0, first_byte]);
                        match map.entry(first_short) {
                            Entry::Occupied(mut occupied) => {
                                // Duplicate character conversion
                                occupied.insert(value);
                            }
                            Entry::Vacant(vacant) => {
                                vacant.insert(value);
                            }
                        }
                    }
                }
                map
            })
            .get(&input)
            .copied()
            .unwrap_or_default()
    }

    #[inline]
    pub fn get_table(table: u8) -> &'static [u8] {
        match table {
            0x00 => include_bytes!("../conversion_tables/00xx.dat"),

            0x81 => include_bytes!("../conversion_tables/81xx.dat"),
            0x82 => include_bytes!("../conversion_tables/82xx.dat"),
            0x83 => include_bytes!("../conversion_tables/83xx.dat"),
            0x84 => include_bytes!("../conversion_tables/84xx.dat"),
            0x85 => include_bytes!("../conversion_tables/85xx.dat"),
            0x86 => include_bytes!("../conversion_tables/86xx.dat"),
            0x87 => include_bytes!("../conversion_tables/87xx.dat"),
            0x88 => include_bytes!("../conversion_tables/88xx.dat"),
            0x89 => include_bytes!("../conversion_tables/89xx.dat"),
            0x8A => include_bytes!("../conversion_tables/8Axx.dat"),
            0x8B => include_bytes!("../conversion_tables/8Bxx.dat"),
            0x8C => include_bytes!("../conversion_tables/8Cxx.dat"),
            0x8D => include_bytes!("../conversion_tables/8Dxx.dat"),
            0x8E => include_bytes!("../conversion_tables/8Exx.dat"),
            0x8F => include_bytes!("../conversion_tables/8Fxx.dat"),

            0x90 => include_bytes!("../conversion_tables/90xx.dat"),
            0x91 => include_bytes!("../conversion_tables/91xx.dat"),
            0x92 => include_bytes!("../conversion_tables/92xx.dat"),
            0x93 => include_bytes!("../conversion_tables/93xx.dat"),
            0x94 => include_bytes!("../conversion_tables/94xx.dat"),
            0x95 => include_bytes!("../conversion_tables/95xx.dat"),
            0x96 => include_bytes!("../conversion_tables/96xx.dat"),
            0x97 => include_bytes!("../conversion_tables/97xx.dat"),
            0x98 => include_bytes!("../conversion_tables/98xx.dat"),
            0x99 => include_bytes!("../conversion_tables/99xx.dat"),
            0x9A => include_bytes!("../conversion_tables/9Axx.dat"),
            0x9B => include_bytes!("../conversion_tables/9Bxx.dat"),
            0x9C => include_bytes!("../conversion_tables/9Cxx.dat"),
            0x9D => include_bytes!("../conversion_tables/9Dxx.dat"),
            0x9E => include_bytes!("../conversion_tables/9Exx.dat"),
            0x9F => include_bytes!("../conversion_tables/9Fxx.dat"),

            0xE0 => include_bytes!("../conversion_tables/E0xx.dat"),
            0xE1 => include_bytes!("../conversion_tables/E1xx.dat"),
            0xE2 => include_bytes!("../conversion_tables/E2xx.dat"),
            0xE3 => include_bytes!("../conversion_tables/E3xx.dat"),
            0xE4 => include_bytes!("../conversion_tables/E4xx.dat"),
            0xE5 => include_bytes!("../conversion_tables/E5xx.dat"),
            0xE6 => include_bytes!("../conversion_tables/E6xx.dat"),
            0xE7 => include_bytes!("../conversion_tables/E7xx.dat"),
            0xE8 => include_bytes!("../conversion_tables/E8xx.dat"),
            0xE9 => include_bytes!("../conversion_tables/E9xx.dat"),
            0xEA => include_bytes!("../conversion_tables/EAxx.dat"),
            0xEB => include_bytes!("../conversion_tables/EBxx.dat"),
            0xEC => include_bytes!("../conversion_tables/ECxx.dat"),
            0xED => include_bytes!("../conversion_tables/EDxx.dat"),
            0xEE => include_bytes!("../conversion_tables/EExx.dat"),
            0xEF => include_bytes!("../conversion_tables/EFxx.dat"),

            0xF0 => include_bytes!("../conversion_tables/F0xx.dat"),
            0xF1 => include_bytes!("../conversion_tables/F1xx.dat"),
            0xF2 => include_bytes!("../conversion_tables/F2xx.dat"),
            0xF3 => include_bytes!("../conversion_tables/F3xx.dat"),
            0xF4 => include_bytes!("../conversion_tables/F4xx.dat"),
            0xF5 => include_bytes!("../conversion_tables/F5xx.dat"),
            0xF6 => include_bytes!("../conversion_tables/F6xx.dat"),
            0xF7 => include_bytes!("../conversion_tables/F7xx.dat"),
            0xF8 => include_bytes!("../conversion_tables/F8xx.dat"),
            0xF9 => include_bytes!("../conversion_tables/F9xx.dat"),
            0xFA => include_bytes!("../conversion_tables/FAxx.dat"),
            0xFB => include_bytes!("../conversion_tables/FBxx.dat"),
            0xFC => include_bytes!("../conversion_tables/FCxx.dat"),
            _ => &EMPTY_TABLE,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::conversion_tables::ConversionTable;

    fn get_misc_conversions() -> Vec<((u8, u8), u16)> {
        let mut u16_buffer = [0u16; 2];

        vec![((0x8D, 0x92 / 2), '巧'.encode_utf16(&mut u16_buffer)[0])]
    }

    #[test]
    fn misc_conversions() {
        get_misc_conversions()
            .into_iter()
            .for_each(|((first_byte, second_byte), short)| {
                eprintln!("{:02X}", first_byte);
                eprintln!("{:02X}", second_byte);
                eprintln!("{:04X} {}", short, short);

                eprintln!("{:04X}", u16::from_le_bytes([second_byte, first_byte]));
                eprintln!("{:04X}", u16::from_le_bytes([first_byte, second_byte]));

                assert_eq!(
                    ConversionTable::lookup(first_byte, second_byte),
                    short,
                    "failed lookup"
                );
                assert_eq!(
                    ConversionTable::rev_lookup(short),
                    u16::from_le_bytes([first_byte, second_byte]),
                    "failed reverse lookup"
                );
            });
    }
}
