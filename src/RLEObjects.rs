use crate::Codec::CodecFunctions;

impl RLEEncoder
{
    pub fn new() -> Self 
    {
        return RLEEncoder { dictionary: Vec::new(), };
    }

    pub fn new_from_buffer(buffer: &[u8]) -> Self
    {
        if buffer.is_empty() { return RLEEncoder{ dictionary: Vec::new(), }; }
        let mut list: Vec<(u8, u32)> = Vec::new();
        let mut working_byte: u8 = buffer[0];
        let mut counter: u32 = 1;
        for &byte in &buffer[1..]
        {
            if byte != working_byte
            {
                list.push((working_byte, counter));
                working_byte = byte;
                counter = 1;
            }
            else { counter += 1; }
        }
        list.push((working_byte, counter));
        return RLEEncoder{ dictionary: list, };
    }

    fn expand_rle(&self) -> Vec<u8> 
    {
        let mut result = Vec::new();
        for (byte, count) in &self.dictionary { result.extend(std::iter::repeat(*byte).take(*count as usize)); }
        return result;
    }

    fn serialize_rle(&self) -> Vec<u8>
    {
        let mut dictionary_serialized: Vec<u8> = Vec::<u8>::with_capacity(self.dictionary.len()*5); // 1 byte + 4 bytes per element
        for (byte, count) in &self.dictionary 
        {
            dictionary_serialized.push(*byte);
            dictionary_serialized.extend(&(*count).to_be_bytes());
        }
        return dictionary_serialized;
    }

    fn deserialize_rle(&mut self, data: &[u8]) 
    {
        self.dictionary.clear();
        let mut i = 0;
        while i + 5 <= data.len() 
        {
            let byte = data[i];
            let count = u32::from_be_bytes([data[i+1], data[i+2], data[i+3], data[i+4]]);
            self.dictionary.push((byte, count));
            i += 5;
        }
        if i != data.len() { eprintln!("Warning: extra {} bytes at end of serialized RLE data", data.len() - i); }
    }
}

impl CodecFunctions for RLEEncoder
{
    fn encode(data: &Vec<u8>) -> std::io::Result<Vec<u8>>
    {
        let mut rle: RLEEncoder = RLEEncoder::new_from_buffer(data);
        return Ok(rle.serialize_rle());
    }

    fn decode(encoded_data: &Vec<u8>) -> std::io::Result<Vec<u8>>
    {
        let mut rle: RLEEncoder = RLEEncoder::new();
        rle.deserialize_rle(encoded_data);
        return Ok(rle.expand_rle());
    }
}

pub struct RLEEncoder
{
    dictionary: Vec<(u8, u32)>,
}