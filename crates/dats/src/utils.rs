pub fn rotate_byte(byte: u8, shift_size: usize) -> u8 {
    (byte >> shift_size) | (byte << (8 - shift_size))
}

#[allow(unused)]
pub fn rotate(bytes: &mut [u8], offset: usize, size: usize, shift_size: usize) {
    if shift_size < 1 || shift_size > 8 {
        return;
    }
    for i in 0..size {
        bytes[offset + i] = rotate_byte(bytes[offset + i], shift_size);
    }
}

pub fn rotate_all(bytes: &mut [u8], shift_size: usize) {
    if shift_size < 1 || shift_size > 8 {
        return;
    }
    bytes
        .iter_mut()
        .for_each(|b| *b = rotate_byte(*b, shift_size));
}

#[inline]
pub fn get_nibble(value: u32, pos: u32) -> u8 {
    ((value & (0xF << (pos * 4))) >> (pos * 4)) as u8
}
