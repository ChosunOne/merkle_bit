/// The number of bytes in the key.
pub const KEY_LEN: usize = 32;
/// The number of bytes in the key as a u8.
pub const KEY_LEN_BYTES: u8 = KEY_LEN as u8;
/// The number of bits in the key.
pub const KEY_LEN_BITS: u8 = (KEY_LEN_BYTES as u16 * 8 - 1) as u8;
/// These constants are used to quickly calculate the values of log2.
pub const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 8] = [0, 5, 1, 6, 4, 3, 2, 7];