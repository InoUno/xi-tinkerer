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

pub fn get_data_shift_size(bytes: &[u8], offset: usize, size: usize) -> usize {
    if size < 13 {
        return 0;
    }

    let bit_count = u8::count_ones(bytes[offset + 2]) as i32
        - u8::count_ones(bytes[offset + 11]) as i32
        + u8::count_ones(bytes[offset + 12]) as i32;

    match bit_count.abs() % 5 {
        0 => 7,
        1 => 1,
        2 => 6,
        3 => 2,
        4 => 5,
        _ => 0,
    }
}

pub fn decode_data_block(bytes: &mut [u8]) {
    rotate_all(bytes, get_data_shift_size(bytes, 0, bytes.len()));
}

pub fn encode_data_block(bytes: &mut [u8]) {
    rotate_all(bytes, 8 - get_data_shift_size(bytes, 0, bytes.len()));
}

pub fn decode_data_block_masked(bytes: &mut [u8]) {
    let save2 = bytes[2];
    let save11 = bytes[11];
    let save12 = bytes[12];
    rotate_all(bytes, get_data_shift_size(bytes, 0, bytes.len()));
    bytes[2] = save2;
    bytes[11] = save11;
    bytes[12] = save12;
}

pub fn encode_data_block_masked(bytes: &mut [u8]) {
    let save2 = bytes[2];
    let save11 = bytes[11];
    let save12 = bytes[12];
    rotate_all(bytes, 8 - get_data_shift_size(bytes, 0, bytes.len()));
    bytes[2] = save2;
    bytes[11] = save11;
    bytes[12] = save12;
}

#[inline]
pub fn get_nibble(value: u32, pos: u32) -> u8 {
    ((value & (0xF << (pos * 4))) >> (pos * 4)) as u8
}
