#[macro_export]
macro_rules! name_bytes {
    ($modname:ident,
        $( $byte:expr => $tag:expr, )*
    ) => {
        pub(crate) mod $modname {
            #[inline]
            pub(crate) fn decode<'a>(input: u8) -> &'static str {
                match input {
                    $( $byte => $tag, )*
                    _ => ""
                }
            }

            #[inline]
            pub(crate) fn encode(input: &str) -> Option<u8> {
                match input {
                    $( $tag => Some($byte), )*
                    _ => None
                }
            }
        }
    };
}

name_bytes!(base_len_1,
    0x0A => "number",
    0x0C => "choice",
    0x0E => "sound",
    0x11 => "spell",
    0x18 => "player",
    0x19 => "item",
    0x1A => "skill",
    0x1C => "entity",
    0x1E => "color",
    0x1F => "color-alt",
);

name_bytes!(prefix_7f_len_1,
    0x34 => "wait-animation",
    0x35 => "wait-35",
    0x36 => "wait-36",
    0x80 => "lettercase",
    0x8D => "weather-event",
    0x8E => "weather-type",
    0x92 => "choice-plurality",
    0x94 => "number-alt",
    0xB1 => "title-alt",
    0xA0 => "ts-year",
    0xA1 => "ts-month",
    0xA2 => "ts-day",
    0xA3 => "ts-hour",
    0xA9 => "ts-minute",
    0xAA => "ts-second",
    0xAB => "earthtime",
    0xAC => "vanatime",
);

name_bytes!(prefix_01,
    0x01 => "article",
    0x03 => "item-count",
    0x12 => "title",
    0x17 => "weather-adjective",
    0x18 => "weather-noun",
    0x23 => "item-singular",
    0x24 => "item-article",
    0x25 => "item-plural",
    0x29 => "item-given-plurality",
    0x33 => "keyitem-singular",
    0x35 => "keyitem-plural",
    0x36 => "keyitem-article",
    0x38 => "zone",
    0x83 => "mission",
);

name_bytes!(icon,
    // Elements
    0x1F => "fire",
    0x20 => "ice",
    0x21 => "wind",
    0x22 => "earth",
    0x23 => "lightning",
    0x24 => "water",
    0x25 => "light",
    0x26 => "dark",

    // Auto-translate braces
    0x27 => "at-open",
    0x28 => "at-close",

    // On/off
    0x29 => "on",
    0x2A => "off",
    0x2B => "oui",
    0x2C => "non",
    0x2D => "ein",
    0x2E => "aus",
);
