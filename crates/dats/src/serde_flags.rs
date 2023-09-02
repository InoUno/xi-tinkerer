use std::{borrow::Cow, fmt};

use anyhow::anyhow;
use bitflags::{
    parser::{ParseHex, WriteHex},
    Flags,
};
use serde::{
    de::{Error, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

pub fn serialize<B: Flags + Serialize, S: Serializer>(
    flags: &B,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    B::Bits: WriteHex + Serialize,
{
    // Serialize human-readable flags as a sequence like `"[A, B]"`
    if serializer.is_human_readable() {
        let mut seq = serializer.serialize_seq(None)?;
        for flag in flags.iter_names() {
            seq.serialize_element(flag.0)?;
        }
        seq.end()
    }
    // Serialize non-human-readable flags directly as the underlying bits
    else {
        flags.bits().serialize(serializer)
    }
}

/**
Deserialize a set of flags from a human-readable string or their underlying bits.

Any unknown bits will be retained.
*/
pub fn deserialize<'de, B: Flags + Default, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<B, D::Error>
where
    B::Bits: ParseHex + Deserialize<'de>,
{
    if deserializer.is_human_readable() {
        // Deserialize human-readable flags by parsing them from sequences like `"[A, B]"`
        struct FlagsVisitor<B>(core::marker::PhantomData<B>);

        impl<'de, B: Flags + Default> Visitor<'de> for FlagsVisitor<B>
        where
            B::Bits: ParseHex,
        {
            type Value = B;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence of flags")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut result = Self::Value::default();
                while let Some(element) = seq.next_element::<Cow<'_, str>>()? {
                    result.insert(
                        B::from_name(&element)
                            .ok_or_else(|| anyhow!("Unknown element: {}", element))
                            .map_err(|e| A::Error::custom(e))?,
                    );
                }
                Ok(result)
            }
        }

        deserializer.deserialize_seq(FlagsVisitor(Default::default()))
    } else {
        // Deserialize non-human-readable flags directly from the underlying bits
        let bits = B::Bits::deserialize(deserializer)?;

        Ok(B::from_bits_retain(bits))
    }
}

#[macro_export]
macro_rules! serde_bitflags {
    ($($name:ident),*$(,)?) => {
        $(
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                crate::serde_flags::serialize(self, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                crate::serde_flags::deserialize(deserializer)
            }
        })*
    };
}

#[cfg(test)]
mod tests {

    use serde_yaml;

    bitflags::bitflags! {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
        pub struct SomeFlags: u16 {
            // Combined flags
            const SecondAndThird = 0x06;

            // Base flags
            const None = 0x00;
            const First = 0x01;
            const Second = 0x02;
            const Third = 0x04;
        }
    }
    serde_bitflags!(SomeFlags);

    #[test]
    pub fn base_flags() {
        assert_eq!(
            "- First\n- Second\n",
            serde_yaml::to_string(&(SomeFlags::First | SomeFlags::Second)).unwrap(),
        );
        assert_eq!(
            SomeFlags::First | SomeFlags::Second,
            serde_yaml::from_str(&"- First\n- Second\n").unwrap(),
        );
    }

    #[test]
    pub fn combined_flags() {
        assert_eq!(
            "- SecondAndThird\n",
            serde_yaml::to_string(&(SomeFlags::Second | SomeFlags::Third)).unwrap(),
        );
        assert_eq!(
            SomeFlags::SecondAndThird,
            serde_yaml::from_str(&"- Second\n- Third\n").unwrap(),
        );
    }
}
