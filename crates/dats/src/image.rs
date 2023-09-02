#![allow(dead_code)] // TODO: remove once images are in use
use anyhow::{anyhow, Result};
use common::byte_walker::ByteWalker;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct XiImage {
    id: String,
    category: String,
    width: u32,
    height: u32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    image_size: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    used_colors: u32,
    important_colors: u32,
}

pub enum XiImageType {
    DirectX,
    BitmapA,
    BitmapB,
}

impl XiImageType {
    pub(crate) fn from(byte: u8) -> Result<Self> {
        match byte {
            0x91 => Ok(XiImageType::BitmapA),
            0xA1 => Ok(XiImageType::DirectX),
            0xB1 => Ok(XiImageType::BitmapB),
            _ => Err(anyhow!("Unknown image type: {:02X}", byte)),
        }
    }
}

impl XiImage {
    pub(crate) fn parse<T: ByteWalker>(walker: &mut T) -> Result<Self> {
        let image_type = XiImageType::from(walker.step::<u8>()?)?;

        let category = std::str::from_utf8(walker.take_bytes(8)?)?.to_string();
        let id = std::str::from_utf8(walker.take_bytes(8)?)?.to_string();

        walker.expect_msg(40, "BITMAPINFO structure length")?;

        let width = walker.step::<u32>()?;
        let height = walker.step::<u32>()?;
        let planes = walker.step::<u16>()?;
        let bit_count = walker.step::<u16>()?;
        let compression = walker.step::<u32>()?;
        let image_size = walker.step::<u32>()?;
        let horizontal_resolution = walker.step::<u32>()?;
        let vertical_resolution = walker.step::<u32>()?;
        let used_colors = walker.step::<u32>()?;
        let important_colors = walker.step::<u32>()?;

        if width > 16 * 1024 || height > 16 * 1024 || planes != 1 {
            return Err(anyhow!("Incompatible width, height, or planes in image."));
        }

        match image_type {
            XiImageType::DirectX => Self::parse_directx(walker, width, height)?,
            XiImageType::BitmapA => todo!(),
            XiImageType::BitmapB => todo!(),
        }

        Ok(XiImage {
            id,
            category,
            width,
            height,
            planes,
            bit_count,
            compression,
            image_size,
            horizontal_resolution,
            vertical_resolution,
            used_colors,
            important_colors,
        })
    }

    fn parse_directx<T: ByteWalker>(walker: &mut T, width: u32, height: u32) -> Result<()> {
        let four_cc = std::str::from_utf8(walker.take_bytes(4)?)?;

        if !four_cc.starts_with("DXT") {
            return Err(anyhow!(
                "Expected first three bytes to be \"DXT\", but got \"{}\"",
                &four_cc[..3]
            ));
        }

        if width % 4 != 0 || height % 4 != 0 {
            return Err(anyhow!(
                "Expected width and height to be a multiple of 4, but got {} and {}.",
                width,
                height
            ));
        }

        let _texel_block_count = height / 4 * width / 4;
        let _unknown1 = walker.step::<u64>()?;

        todo!();
    }
}
