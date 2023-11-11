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
    0x05 => "sys-msg-3",
    0x0A => "number",
    0x0C => "choice",
    0x0E => "sound",
    0x10 => "spell-alt",
    0x11 => "spell",
    0x12 => "number-alt",
    0x14 => "countdown-seconds",
    0x16 => "skill-alt",
    0x17 => "animation",
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
    0x84 => "unknown-84",
    0x86 => "choice-plurality-number",
    0x87 => "choice-plurality-entity",
    0x88 => "choice-definite-entity",
    0x8D => "weather-event",
    0x8E => "weather-type",
    0x8F => "ability",
    0x92 => "choice-plurality",
    0x94 => "number-2-digits",
    0xA0 => "ts-year",
    0xA1 => "ts-month",
    0xA2 => "ts-day",
    0xA3 => "ts-hour",
    0xA9 => "ts-minute",
    0xAA => "ts-second",
    0xAB => "earthtime",
    0xAC => "vanatime",
    0xB1 => "title-alt",
    0xB4 => "gil",
);

name_bytes!(prefix_01,
    0x01 => "article",
    0x03 => "item-count",
    0x04 => "item-count-alt",
    0x10 => "entity-source",
    0x11 => "entity-target",
    0x12 => "title",
    0x13 => "status-effect-noun",
    0x14 => "status-effect-adjective",
    0x17 => "weather-adjective",
    0x18 => "weather-noun",
    0x23 => "item-singular",
    0x24 => "item-article",
    0x25 => "item-plural",
    0x26 => "item-singular-alt",
    0x27 => "item-article-alt",
    0x28 => "item-plural-alt",
    0x29 => "item-given-plurality",
    0x2A => "item-given-plurality-alt",
    0x33 => "keyitem-singular",
    0x35 => "keyitem-plural",
    0x36 => "keyitem-article",
    0x38 => "zone",
    0x41 => "roe",
    0x42 => "keyitem",
    0x45 => "keyitem-with-article",
    0x83 => "mission",
    0x84 => "chocobo-name",
    0x85 => "choice-chocobo-gender",
    0x86 => "chocobo-word",
    0x88 => "ally-dialog",
    0x89 => "unity",
    0x8A => "augment",
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
