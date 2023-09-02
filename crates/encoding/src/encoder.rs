use std::{iter::Peekable, str::CharIndices};

use crate::{
    conversion_tables::ConversionTable,
    named_bytes::{base_len_1, icon, prefix_01, prefix_7f_len_1},
    TAG_END, TAG_PARAM_START, TAG_PREFIX, TAG_START,
};
use anyhow::{anyhow, Result};

pub struct Encoder<'a> {
    decoded_bytes: Vec<u8>,
    source_str: &'a str,
    source_chars: Peekable<CharIndices<'a>>,
    had_prompt: bool,
}

impl<'a> Encoder<'a> {
    pub fn new(str: &'a str) -> Self {
        Self {
            decoded_bytes: vec![],
            source_str: str,
            source_chars: str.char_indices().peekable(),
            had_prompt: false,
        }
    }
}

impl<'a> Encoder<'a> {
    pub fn encode_simple(string: &'a str) -> Result<Vec<u8>> {
        Self::encode(string, true)
    }

    pub fn encode_dialog(string: &'a str) -> Result<Vec<u8>> {
        Self::encode(string, false)
    }

    pub(crate) fn encode(string: &'a str, is_simple: bool) -> Result<Vec<u8>> {
        let mut encoder = Encoder::new(string);
        if is_simple {
            encoder.encode_all::<true>()?;
        } else {
            encoder.encode_all::<false>()?;
        }

        // Add end marker if needed
        if !is_simple {
            // Insert 0x00 if there's not been a prompt, and it doesn't
            // already have one at the end (past any whitespaces).
            if !encoder.had_prompt {
                let ending_ws_idx = string.len()
                    - string
                        .chars()
                        .rev()
                        .take_while(|ch| ch.is_whitespace())
                        .map(|ch| ch.len_utf8())
                        .sum::<usize>();

                if ending_ws_idx == 0 || string[..ending_ws_idx].chars().rev().next() != Some('\0')
                {
                    encoder.decoded_bytes.push(0x00);
                }
            }

            encoder.decoded_bytes.push(0x07);
        }

        Ok(encoder.decoded_bytes)
    }

    fn encode_all<const IS_SIMPLE: bool>(&mut self) -> Result<()> {
        let mut u16_buffer = [0u16; 2];

        while let Some((_, char)) = self.source_chars.next() {
            match char {
                TAG_PREFIX => {
                    // Check if it's a tag
                    if self
                        .source_chars
                        .peek()
                        .map(|(_, ch)| *ch == TAG_START)
                        .unwrap_or_default()
                    {
                        // Skip past the TAG_START and handle the tag
                        self.source_chars.next();
                        self.handle_tag()?;
                        continue;
                    }
                }
                '\n' => {
                    self.decoded_bytes.push(if IS_SIMPLE { 0x0A } else { 0x07 });
                    continue;
                }
                '\0' => {
                    self.decoded_bytes.push(0x00);
                    continue;
                }
                _ => {}
            }

            // Lookup UTF-16 character in the reverse conversion table
            let original_utf16_shorts = char.encode_utf16(&mut u16_buffer);
            let converted_utf16_shorts = original_utf16_shorts
                .iter()
                .map(|short| ConversionTable::rev_lookup(*short))
                .collect::<Vec<_>>();

            let final_utf16_shorts = if converted_utf16_shorts.iter().all(|short| *short > 0) {
                converted_utf16_shorts.as_ref()
            } else {
                original_utf16_shorts.as_ref()
            };

            // Convert UTF-16 characters to bytes
            let u8_bytes = final_utf16_shorts
                .iter()
                .map(|u16| u16.to_le_bytes())
                .flatten()
                .filter(|byte| *byte != 0);
            self.decoded_bytes.extend(u8_bytes);
        }

        Ok(())
    }

    #[inline]
    fn parse_tag(&mut self) -> Result<(&'a str, &'a str)> {
        let tag_start = self
            .source_chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or_default();

        // Parse tag name
        let mut tag_end = tag_start;
        while let Some((idx, char)) = self.source_chars.next() {
            tag_end = idx;

            if char == TAG_END {
                // Tag closed without params
                return Ok((&self.source_str[tag_start..tag_end], &""));
            } else if char == TAG_PARAM_START {
                // Tag indicates it has params
                break;
            }
        }

        // Parse the params of the tag
        let mut params_start = tag_end + 1;

        // Skip past any starting spaces
        while let Some((_, ' ')) = self.source_chars.peek() {
            params_start += 1;
            self.source_chars.next();
        }

        // Parse content extent and tag closing character
        let mut params_end = params_start;
        while let Some((idx, char)) = self.source_chars.next() {
            params_end = idx;
            if char == TAG_END {
                break;
            }
        }

        let res = Ok((
            &self.source_str[tag_start..tag_end],
            &self.source_str[params_start..params_end],
        ));

        res
    }

    fn handle_tag(&mut self) -> Result<()> {
        // Just got a TAG_START char
        let (tag, content) = self.parse_tag()?;

        match tag {
            "prompt" => {
                self.decoded_bytes.extend([0x7F, 0x31, 0x00]);
                self.had_prompt = true;
                return Ok(());
            }

            "selection-lines" => {
                self.decoded_bytes.push(0x0B);
                // Skip one following newline if there is one, since it's inserted
                // for increased readability during decoding.
                if let Some((_, '\n')) = self.source_chars.peek() {
                    self.source_chars.next();
                }
                return Ok(());
            }

            "name-player" => {
                self.decoded_bytes.push(0x08);
                return Ok(());
            }

            "name-npc" => {
                self.decoded_bytes.push(0x09);
                return Ok(());
            }

            "icon" => {
                self.decoded_bytes.push(0xEF);

                if let Some(icon_byte) = icon::encode(content) {
                    self.decoded_bytes.push(icon_byte);
                } else {
                    let bytes = (2..content.len())
                        .step_by(2)
                        .map(|idx| u8::from_str_radix(&content[idx..idx + 2], 16).unwrap());

                    self.decoded_bytes.extend(bytes);
                }

                return Ok(());
            }

            "choice-player-gender" => {
                self.decoded_bytes.extend([0x7F, 0x85]);
                return Ok(());
            }

            "related-entity" => {
                self.decoded_bytes.extend([0x7F, 0x93]);
                return Ok(());
            }

            "unknown" | "unknown-table" | "unknown-table-index" | "unknown-table-value" => {
                let bytes = (2..content.len())
                    .step_by(2)
                    .map(|idx| u8::from_str_radix(&content[idx..idx + 2], 16).unwrap());

                self.decoded_bytes.extend(bytes);
                return Ok(());
            }

            _ => {}
        }

        if let Some(byte) = base_len_1::encode(tag) {
            self.decoded_bytes.extend(&[byte, content.parse()?]);

            //
        } else if let Some(byte) = prefix_7f_len_1::encode(tag) {
            let Ok(param) = content.parse::<u8>() else {
                return Err(anyhow!(
                    "Failed to parse parameter '{}' at '{}'",
                    content,
                    tag
                ));
            };

            self.decoded_bytes.extend(&[0x7F, byte, param]);

            //
        } else if let Some(byte) = prefix_01::encode(tag) {
            if content.len() == 0 {
                self.decoded_bytes.extend(&[0x01, 0x01, byte]);
            } else {
                let parameters = content
                    .split(',')
                    .filter_map(|param| {
                        let parsed_param = Self::parse_param_with_length(param);
                        if parsed_param.is_none() {
                            eprintln!("Could not parse param of tag '{}': {}", tag, param);
                        }
                        parsed_param
                    })
                    .collect::<Vec<_>>();

                let parameters_len: u32 = parameters.iter().map(|(_, size)| size + 2).sum();

                let param_bytes = parameters
                    .into_iter()
                    .map(|(value, len)| {
                        std::iter::once(len as u8)
                            .chain(value.to_le_bytes()[..len as usize].iter().copied())
                            .chain(std::iter::once(0))
                            .map(|byte| byte ^ 0x80)
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                self.decoded_bytes
                    .extend(&[0x01, 1 + parameters_len as u8, byte]);
                self.decoded_bytes.extend(param_bytes);
            }
        } else {
            // Unhandled tag
            eprintln!("Unhandled tag and content: {} {}", tag, content);
        }

        Ok(())
    }

    fn parse_param_with_length(param: &str) -> Option<(u32, u32)> {
        let mut param_iter = param.char_indices().peekable();

        // Skip whitespaces
        while let Some((_, ch)) = param_iter.peek() {
            if !ch.is_whitespace() {
                break;
            }
            param_iter.next();
        }

        // Extract first value
        let value_start_idx = param_iter.next()?.0;
        while let Some((_, ch)) = param_iter.peek() {
            if !ch.is_numeric() {
                break;
            }
            param_iter.next();
        }
        let value_end_idx = param_iter
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or_else(|| param.len());

        let value = param[value_start_idx..value_end_idx].parse::<u32>().ok()?;

        // Skip more whitespaces
        while let Some((_, ch)) = param_iter.peek() {
            if !ch.is_whitespace() {
                break;
            }
            param_iter.next();
        }

        // Expect opening bracket
        if let Some((_, ch)) = param_iter.next() {
            if ch != '[' {
                return None;
            }
        }

        // Extract value byte length
        let len_start_idx = param_iter.next()?.0;
        while let Some((_, ch)) = param_iter.peek() {
            if !ch.is_numeric() {
                break;
            }
            param_iter.next();
        }
        let len_end_idx = param_iter
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or_else(|| param.len());

        let len = param[len_start_idx..len_end_idx].parse::<u32>().ok()?;

        // Expect closing bracket
        if let Some((_, ch)) = param_iter.next() {
            if ch != ']' {
                return None;
            }
        }

        Some((value, len))
    }
}

#[cfg(test)]
mod tests {

    use crate::{decoder::Decoder, encoder::Encoder, tests::example_strings_for_encoding};
    use pretty_assertions::assert_eq;

    #[test]
    fn example_translations() {
        for (bytes, string) in example_strings_for_encoding() {
            assert_eq!(Encoder::encode_dialog(string).unwrap(), bytes);
        }
    }

    fn check_encoding_and_roundtrip(bytes: &[u8], string: &str, message: &str) {
        assert_eq!(
            Encoder::encode_dialog(string).unwrap(),
            bytes,
            "{}",
            message
        );
        assert_eq!(
            Encoder::encode_dialog(&Decoder::decode_dialog(&bytes).unwrap()).unwrap(),
            bytes,
            "{} (round-trip)",
            message
        );
    }

    #[test]
    fn ending_characters() {
        check_encoding_and_roundtrip(&[0x00, 0x07], "", "empty string");

        check_encoding_and_roundtrip(
            &[0x21, 0x07, 0x20, 0x00, 0x07],
            "!\n ",
            "newlines and spaces",
        );

        check_encoding_and_roundtrip(
            &[0x81, 0x40, 0x00, 0x07],
            "\u{3000}",
            "just invisible space",
        );

        check_encoding_and_roundtrip(&[0x20, 0x20, 0x20, 0x00, 0x07], "   ", "triple space");

        check_encoding_and_roundtrip(
            &[0x7F, 0x31, 0x00, 0x20, 0x07],
            "${prompt} ",
            "prompt and space",
        );

        check_encoding_and_roundtrip(&[0x7F, 0x31, 0x00, 0x07], "${prompt}", "prompt");
    }
}
