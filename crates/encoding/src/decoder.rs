use std::char::decode_utf16;

use crate::{
    conversion_tables::ConversionTable,
    encoder::Encoder,
    named_bytes::{base_len_1, icon, prefix_01, prefix_7f_len_1},
    SPACE_U16, TAG_END_U16, TAG_PARAM_START_U16, TAG_PREFIX_U16, TAG_START_U16,
};
use anyhow::Result;

pub struct Decoder<'a> {
    decoded_bytes: Vec<u8>,
    source_bytes: &'a [u8],
    idx: usize,
    end_idx: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8], _is_simple: bool) -> Self {
        Self {
            decoded_bytes: vec![],
            source_bytes: bytes,
            idx: 0,
            end_idx: bytes.len(),
        }
    }
}

impl<'a> Decoder<'a> {
    pub fn decode_simple(bytes: &[u8]) -> Result<String> {
        Self::decode(bytes, true)
    }

    pub fn decode_dialog(bytes: &[u8]) -> Result<String> {
        Self::decode(bytes, false)
    }

    pub(crate) fn decode(bytes: &[u8], is_simple: bool) -> Result<String> {
        if bytes.is_empty() {
            return Ok("".to_string());
        }

        let mut decoder = Decoder::new(bytes, is_simple);
        if is_simple {
            decoder.decode_all::<true>();
        } else {
            decoder.decode_all::<false>();
        }

        // Convert the decoded bytes to a string via UTF-16
        let len = decoder.decoded_bytes.len() / 2;
        let iter = (0..len).map(|i| {
            u16::from_be_bytes([
                decoder.decoded_bytes[2 * i],
                decoder.decoded_bytes[2 * i + 1],
            ])
        });

        let mut string = decode_utf16(iter).collect::<Result<String, _>>()?;
        if !is_simple {
            string = string.trim_end_matches(&['\n', '\0']).to_string();
        }

        // Verify re-encoding in debug mode
        if false && cfg!(debug_assertions) {
            // Figure out last index of the bytes, based on what it's expected to end with.
            let last_idx: usize = if is_simple {
                bytes
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(idx, byte)| if *byte != 0x00 { Some(idx + 1) } else { None })
                    .unwrap_or(0)
            } else {
                bytes
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(idx, byte)| if *byte == 0x07 { Some(idx) } else { None })
                    .unwrap_or(bytes.len() - 1)
                    + 1
            };

            let bytes = &bytes[..last_idx];

            let encoded_bytes = Encoder::encode(&string, is_simple)?;

            if encoded_bytes != bytes {
                eprintln!(
                    "\n=====\nMismatch:\n{:?}\n\nOriginal: {:02X?}\nEncoded:  {:02X?}",
                    string, bytes, encoded_bytes
                );
            }
        }

        Ok(string)
    }

    fn decode_all<const IS_SIMPLE: bool>(&mut self) {
        while self.idx < self.end_idx {
            let byte = self.get_at_offset(0);

            // Cases that extend by exactly 1 byte
            if !IS_SIMPLE && self.can_extend(1) {
                let tag = base_len_1::decode(byte);
                if tag != "" {
                    self.make_byte_tag(tag, self.get_at_offset(1));
                    self.idx += 2;
                    continue;
                }
            }

            if IS_SIMPLE && byte == 0x0A {
                self.push_str("\n");
                self.idx += 1;
                continue;
            }

            if byte == 0xFD && self.can_extend(5) && self.get_at_offset(5) == 0xFD {
                // TODO: Resource bytes?
                self.make_hex_bytes_tag("resource", &self.source_bytes[self.idx..self.idx + 5]);
                self.idx += 6;
                continue;
            }

            match byte {
                0x00 => {
                    // Check for ending sequence
                    if IS_SIMPLE || self.can_extend(1) && self.get_at_offset(1) == 0x07 {
                        break;
                    }
                    self.decoded_bytes.push(byte);
                    self.decoded_bytes.push(byte);
                    self.idx += 1;
                }

                0x01 if !IS_SIMPLE => {
                    self.idx += 1;
                    self.decode_01();
                }

                0x02 => {
                    // Is usually at the start of a string. It removes the UI when used as a raw message in-game.
                    self.make_hex_bytes_tag("unknown", &self.source_bytes[self.idx..self.idx + 5]);
                    self.idx += 5;
                }

                0x07 => {
                    self.push_str("\n");
                    self.idx += 1;
                }

                0x08 => {
                    self.tag_no_params("name-player");
                    self.idx += 1;
                }

                0x09 => {
                    self.tag_no_params("name-npc");
                    self.idx += 1;
                }

                0x0B => {
                    self.tag_no_params("selection-lines");
                    self.push_str("\n");
                    self.idx += 1;
                }

                0xEF => {
                    self.idx += 1;
                    self.decode_ef();
                }

                0x7F if !IS_SIMPLE => {
                    self.idx += 1;
                    self.decode_7f();
                }

                0..=0x19 => {
                    // This range seems to always extend one byte,
                    // so include it if possible.
                    self.tag_open_params("unknown");
                    if self.can_extend(1) {
                        self.push_hex(&self.source_bytes[self.idx..self.idx + 2]);
                        self.idx += 2;
                    } else {
                        self.push_hex_byte(byte);
                        self.idx += 1;
                    }
                    self.tag_close();
                }

                _ => {
                    // Default to conversion table lookup
                    self.table_decode();
                }
            }
        }
    }

    fn decode_7f(&mut self) {
        let byte = self.get_at_offset(0);

        if self.can_extend(1) {
            let tag = prefix_7f_len_1::decode(byte);
            if tag != "" {
                self.make_byte_tag(tag, self.get_at_offset(1));
                self.idx += 2;
                return;
            }
        }

        match byte {
            0x31 => {
                self.tag_no_params("prompt");

                // Prompts are seemingly always followed by a 0x00 byte.
                if !self.can_extend(1) || self.get_at_offset(1) != 0x00 {
                    eprintln!("Expected zero-byte after prompt (0x7F31).");
                }
                self.idx += 2;

                // End immediately if prompt is followed by a newline
                if self.can_extend(0) && self.get_at_offset(0) == 0x07 {
                    self.idx = self.end_idx;
                }
            }

            0x38 if self.can_extend(2) => {
                self.make_hex_bytes_tag("unknown", &self.source_bytes[self.idx - 1..self.idx + 3]);
                self.idx += 3;
            }

            0x85 => {
                self.tag_no_params("choice-player-gender");
                self.idx += 1;
            }

            0x90 => {
                self.tag_no_params("choice-source-gender");
                self.idx += 1;
            }

            0x91 => {
                self.tag_no_params("choice-target-gender");
                self.idx += 1;
            }

            0x93 => {
                // Example uses:
                //      <unknown>0x7F93</unknown> has entered the hostel.<prompt>0</prompt>
                //      Hi there! I'm <unknown>0x7F93</unknown>, your friendly neighborhood smile sergeant!
                self.tag_no_params("related-entity");
                self.idx += 1;
            }

            0xFB => {
                self.tag_no_params("entity-wrap-end");
                self.idx += 1;
            }

            0xFC => {
                self.tag_no_params("entity-wrap-start");
                self.idx += 1;
            }

            _ if self.can_extend(1) => {
                self.make_hex_bytes_tag("unknown", &self.source_bytes[self.idx - 1..self.idx + 2]);
                self.idx += 2;
            }

            _ => {
                self.make_hex_bytes_tag("unknown", &self.source_bytes[self.idx - 1..self.idx + 1]);
                self.idx += 1;
            }
        }
    }

    // 0x01 blocks start with a length of the block, and then sub-blocks with endings inside it.
    // After the length, the following bytes are XOR'd with 0x80.
    // Example: [0x01, 0x08, 0x83,  0x81, 0x85, 0x80, 0x82, 0x93, 0x80, 0x80]
    //           ^id   ^len  ^type   ^     ^     ^end  ^2nd sub-block start
    //                               |  1st value
    //                       len of sub-block
    fn decode_01(&mut self) {
        let len = self.get_at_offset(0);
        self.idx += 1;

        if self.can_extend(len as usize) {
            let tag = prefix_01::decode(self.get_at_offset(0));

            if tag != "" {
                let block_bytes = &self.source_bytes[self.idx + 1..self.idx + len as usize];
                let mut values: Vec<String> = vec![];
                let mut idx = 0;

                // Parse sub blocks
                while idx < block_bytes.len() {
                    let sub_len = block_bytes[idx] ^ 0x80;
                    idx += 1;
                    let sub_block_bytes = match sub_len {
                        1 => [block_bytes[idx] ^ 0x80, 0, 0, 0],
                        2 => [block_bytes[idx] ^ 0x80, block_bytes[idx + 1] ^ 0x80, 0, 0],
                        4 => [
                            block_bytes[idx] ^ 0x80,
                            block_bytes[idx + 1] ^ 0x80,
                            block_bytes[idx + 2] ^ 0x80,
                            block_bytes[idx + 3] ^ 0x80,
                        ],
                        _ => {
                            eprintln!("Invalid sub block length {}", sub_len);
                            [0, 0, 0, 0]
                        }
                    };
                    let value = u32::from_le_bytes(sub_block_bytes);
                    idx += sub_len as usize + 1; // Skip closing bytes as well.
                    values.push(format!("{}[{}]", value, sub_len));
                }

                if values.is_empty() {
                    self.tag_no_params(tag);
                } else {
                    self.make_str_tag(tag, &values.join(", "));
                }
            } else {
                self.make_hex_bytes_tag(
                    "unknown",
                    &self.source_bytes[self.idx - 2..self.idx + len as usize],
                );
            }

            self.idx += len as usize;
        } else {
            self.make_hex_bytes_tag("unknown", &self.source_bytes[self.idx - 2..self.idx + 1]);
            self.idx += 1;
        }
    }

    // Icons
    fn decode_ef(&mut self) {
        let icon_name = icon::decode(self.get_at_offset(0));
        if icon_name != "" {
            self.make_str_tag("icon", icon_name);
            self.idx += 1;
        } else {
            self.make_hex_bytes_tag("icon", &self.source_bytes[self.idx - 1..self.idx + 1]);
            self.idx += 1;
        }
    }

    #[inline]
    fn can_extend(&self, amount: usize) -> bool {
        self.idx + amount < self.end_idx
    }

    #[inline]
    fn get_at_offset(&self, offset: usize) -> u8 {
        self.source_bytes[self.idx + offset]
    }

    #[inline]
    fn tag_open_params(&mut self, tag_name: &str) {
        self.tag_open(tag_name);
        self.decoded_bytes.extend(TAG_PARAM_START_U16);
        self.decoded_bytes.extend(SPACE_U16);
    }

    #[inline]
    fn tag_open(&mut self, tag_name: &str) {
        self.decoded_bytes.extend(TAG_PREFIX_U16);
        self.decoded_bytes.extend(TAG_START_U16);
        self.push_str(tag_name);
    }

    #[inline]
    fn tag_close(&mut self) {
        self.decoded_bytes.extend(TAG_END_U16);
    }

    #[inline]
    fn tag_no_params(&mut self, tag_name: &str) {
        self.tag_open(tag_name);
        self.tag_close();
    }

    #[inline]
    fn make_byte_tag(&mut self, tag: &str, byte: u8) {
        self.tag_open_params(tag);
        self.push_byte(byte);
        self.tag_close();
    }

    #[inline]
    fn make_hex_byte_tag(&mut self, tag: &str, byte: u8) {
        self.tag_open_params(tag);
        self.push_hex_byte(byte);
        self.tag_close();
    }

    #[inline]
    fn make_hex_bytes_tag(&mut self, tag: &str, bytes: &[u8]) {
        self.tag_open_params(tag);
        self.push_hex(bytes);
        self.tag_close();
    }

    #[inline]
    fn make_str_tag(&mut self, tag: &str, str: &str) {
        self.tag_open_params(tag);
        self.push_str(str);
        self.tag_close();
    }

    #[inline]
    fn push_str(&mut self, str: &str) {
        self.decoded_bytes.extend(
            str.encode_utf16()
                .map(|codepoint| codepoint.to_be_bytes())
                .flatten(),
        );
    }

    #[inline]
    fn push_hex(&mut self, bytes: &[u8]) {
        self.push_str("0x");
        for byte in bytes {
            self.push_str(&format!("{:02X}", byte.clone()));
        }
    }

    #[inline]
    fn push_hex_byte(&mut self, byte: u8) {
        self.push_str(&format!("0x{:02X}", byte));
    }

    #[inline]
    fn push_byte(&mut self, byte: u8) {
        self.push_str(&format!("{}", byte));
    }

    fn table_decode(&mut self) {
        debug_assert!(self.idx < self.end_idx);

        let first_byte = self.source_bytes[self.idx];
        self.idx += 1;

        let primary_table_value = ConversionTable::lookup(0, first_byte);

        if primary_table_value == 0xFFFE {
            if self.idx >= self.end_idx {
                self.make_hex_byte_tag("unknown-table-index", first_byte);
                eprintln!("Missing index table: 0x{:02X}", first_byte);
                return;
            }

            // Look up second byte in the conversion table of the first byte
            let second_byte = self.source_bytes[self.idx];
            self.idx += 1;

            let secondary_table_value = ConversionTable::lookup(first_byte, second_byte);

            if secondary_table_value == 0xFFFF {
                eprintln!(
                    "Missing table lookup: 0x{:02X} 0x{:02X}",
                    first_byte, second_byte
                );
                self.make_hex_bytes_tag("unknown-table-value", &[first_byte, second_byte]);
            } else {
                // Secondary table decoded character bytes
                self.decoded_bytes
                    .extend(secondary_table_value.to_be_bytes());
            }
        } else if primary_table_value == 0xFFFF {
            // Unknown table lookup
            self.make_hex_byte_tag("unknown-table", first_byte);
        } else {
            // Regular decoded character bytes
            self.decoded_bytes.extend(primary_table_value.to_be_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::example_strings_for_encoding;

    use super::Decoder;

    #[test]
    fn example_translations() {
        for (bytes, string) in example_strings_for_encoding() {
            assert_eq!(Decoder::decode_dialog(&bytes).unwrap(), string);
        }
    }

    #[test]
    fn smoke() {
        let x = '　' as u32;
        let y = unsafe {
            char::from_u32_unchecked(u32::from_le_bytes([0, 0x30, 0, 0])).is_whitespace()
        };
        eprintln!("{:08X} {}", x, y);
    }
}
