mod conversion_tables;
pub mod decoder;
pub mod encoder;
mod named_bytes;

const TAG_PREFIX: char = '$';
const TAG_START: char = '{';
const TAG_END: char = '}';
const TAG_PARAM_START: char = ':';

const TAG_PREFIX_U16: [u8; 2] = (TAG_PREFIX as u16).to_be_bytes();
const TAG_START_U16: [u8; 2] = (TAG_START as u16).to_be_bytes();
const TAG_END_U16: [u8; 2] = (TAG_END as u16).to_be_bytes();
const TAG_PARAM_START_U16: [u8; 2] = (TAG_PARAM_START as u16).to_be_bytes();
const SPACE_U16: [u8; 2] = (' ' as u16).to_be_bytes();

#[cfg(test)]
mod tests {

    pub(crate) fn example_strings_for_encoding() -> Vec<(&'static [u8], &'static str)> {
        vec![
            (&[
                67, 97, 110, 32, 121, 97, 32, 105, 109, 97, 103, 105, 110, 101, 32, 105, 116, 44, 32,
                8, 63, 7, 10, 1, 1, 5, 37, 130, 128, 128, 128, 46, 46, 46, 127, 49, 0, 7,
            ],
            "Can ya imagine it, ${name-player}?\n${number: 1}${item-plural: 0[2]}...${prompt}"
            ),

            (&[
                87, 104, 97, 116, 32, 100, 111, 32, 121, 111, 117, 32, 115, 112, 101, 97, 107, 32, 111,
                102, 63, 7, 11, 82, 97, 122, 102, 97, 104, 100, 39, 115, 32, 109, 101, 115, 115, 97,
                103, 101, 46, 7, 82, 101, 99, 101, 110, 116, 32, 101, 118, 101, 110, 116, 115, 46, 127, 49, 0, 7
            ],
            "What do you speak of?\n${selection-lines}\nRazfahd's message.\nRecent events.${prompt}"
            ),

            (&[
                83, 112, 101, 97, 107, 32, 121, 111, 117, 114, 32, 100, 101, 115, 105, 114, 101, 46, 46, 46, 7, 11, 127, 128, 1, 1, 1, 1, 32, 1, 5, 36, 130, 128, 128, 128, 46, 7, 127, 128, 1, 1, 1, 1, 32, 1, 5, 36, 130, 129, 128, 128, 46, 7, 127, 128, 1, 1, 1, 1, 32, 1, 5, 36, 130, 130, 128, 128, 46, 7, 67, 111, 108, 100, 44, 32, 104, 97, 114, 100, 32, 103, 105, 108, 46, 7, 65, 32, 112, 97, 99, 116, 32, 119, 105, 116, 104, 32, 65, 108, 101, 120, 97, 110, 100, 101, 114, 46, 127, 49, 0, 7
            ],
            "Speak your desire...\n${selection-lines}\n${lettercase: 1}${article} ${item-article: 0[2]}.\n${lettercase: 1}${article} ${item-article: 1[2]}.\n${lettercase: 1}${article} ${item-article: 2[2]}.\nCold, hard gil.\nA pact with Alexander.${prompt}"
            ),

            (&[131, 84, 129, 91, 131, 111, 129, 91, 145, 164, 130, 197, 130, 183, 129, 66, 7, 142, 99, 130, 232, 131, 76, 131, 109, 131, 82, 143, 138, 142, 157, 144, 148, 130, 205, 10, 1, 150, 123, 129, 65, 7, 150, 129, 130, 162, 130, 189, 137, 241, 144, 148, 130, 205, 10, 0, 137, 241, 130, 201, 130, 200, 130, 193, 130, 196, 130, 162, 130, 220, 130, 183, 129, 66, 127, 49, 0, 7],
            "サーバー側です。\n残りキノコ所持数は${number: 1}本、\n磨いた回数は${number: 0}回になっています。${prompt}"
            )
        ]
    }
}
