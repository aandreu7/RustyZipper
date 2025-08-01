use sha2::{Sha256, Digest};
use sha2::digest::Output;

use crate::Codec::CodecFunctions;

impl CodecFunctions for CaesarEncoder
{
    fn encode(data: &Vec<u8>, key: &i64) -> std::io::Result<Vec<u8>>
    {
        let mut result: Vec<u8> = Vec::<u8>::with_capacity(data.len() + 32);

        let key_bytes: [u8; 8] = key.to_be_bytes();
        let key_hashed: Output<Sha256> = Sha256::digest(&key_bytes);

        result.extend_from_slice(&key_hashed);
        for &byte in data
        {
            result.push(byte.wrapping_add(*key as u8));
        }
        return Ok(result);
    }

    fn decode(encoded_data: &Vec<u8>, possible_key: &i64) -> std::io::Result<Vec<u8>>
    { 
        if encoded_data.len() < 33 { return Ok(Vec::new()); }

        let hashed_key = &encoded_data[..32];
        let possible_key_bytes: [u8; 8] = possible_key.to_be_bytes();
        let possible_key_hash: Output<Sha256> = Sha256::digest(&possible_key_bytes);

        if hashed_key != possible_key_hash.as_slice() { return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Invalid key")); }

        let mut result: Vec<u8> = Vec::with_capacity(encoded_data.len() - 32);
        for &byte in encoded_data.iter().skip(32) { result.push(byte.wrapping_sub(*possible_key as u8)); }
        return Ok(result);
    }
}

struct CaesarEncoder
{
}