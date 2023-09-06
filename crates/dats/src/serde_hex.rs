use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
    let hex = format!(
        "0x{}",
        v.iter().map(|b| format!("{:02X}", b)).collect::<String>()
    );
    String::serialize(&hex, s)
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    let hex = String::deserialize(d)?;
    Ok((2..hex.len())
        .step_by(2)
        .map(|idx| u8::from_str_radix(&hex[idx..idx + 2], 16).unwrap())
        .collect())
}
