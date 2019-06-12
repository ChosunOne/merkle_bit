/// The number of bytes in the key.  Maximum value of 32.
pub const KEY_LEN: usize = 32;
/// The number of bits in the key.
pub const KEY_LEN_BITS: usize = (KEY_LEN * 8 - 1);
/// These constants are used to quickly calculate the values of log2.
pub const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 8] = [0, 5, 1, 6, 4, 3, 2, 7];
