pub mod byte_functions;
pub mod byte_walker;
pub mod checking_byte_walker;
pub mod file_byte_walker;
pub mod vec_byte_walker;
pub mod writing_byte_walker;

use std::{ffi::CStr, fmt::Display};

use anyhow::{anyhow, Result};

// unsafe: s must contain a null byte, and be valid utf-8
pub unsafe fn str_from_null_terminated_utf8_unchecked(s: &[u8]) -> &str {
    std::str::from_utf8_unchecked(CStr::from_ptr(s.as_ptr() as *const _).to_bytes())
}

pub fn expect<T: Eq + Display>(expected: T, gotten: T) -> Result<()> {
    if expected != gotten {
        return Err(anyhow!("Expected {}, found {}", expected, gotten));
    }
    Ok(())
}

pub fn expect_msg<T: Eq + Display>(expected: T, gotten: T, message: impl AsRef<str>) -> Result<()> {
    if expected != gotten {
        return Err(anyhow!(
            "Expected {}, found {}: {}",
            expected,
            gotten,
            message.as_ref()
        ));
    }
    Ok(())
}

#[inline]
pub fn get_padding(len: usize) -> usize {
    (4 - (len & 3)) & 3
}
