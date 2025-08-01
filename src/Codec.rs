use num_enum::TryFromPrimitive;

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum CodecList
{
    RustyZipper,
    Huffman,
    RLE,
    Caesar,
    LZ77,
    Arithmetic,
}

pub trait CodecFunctions
{
    fn encode(data: &Vec<u8>) -> std::io::Result<Vec<u8>>;
    fn decode(encoded_data: &Vec<u8>) -> std::io::Result<Vec<u8>>;
}