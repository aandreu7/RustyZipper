use num_enum::TryFromPrimitive;

use crate::{RZ_KEY_TYPE, KEY_LENGTH_BYTES, KEY_LENGTH_BITS};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CodecList
{
    RustyZipper,
    Huffman,
    RLE,
    Caesar,
    AES,
    LZ77,
    Arithmetic,
}

pub trait CodecFunctions
{
    fn encode(data: &Vec<u8>, key: Option<&RZ_KEY_TYPE>) -> std::io::Result<Vec<u8>>;
    fn decode(encoded_data: &Vec<u8>, key: Option<&RZ_KEY_TYPE>) -> std::io::Result<Vec<u8>>;
}