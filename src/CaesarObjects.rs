use crate::Codec::CodecFunctions;
use crate::HashHandling::{verify_hash, hash_key};
use crate::{RZ_KEY_TYPE, KEY_LENGTH_BYTES, KEY_LENGTH_BITS};

impl CodecFunctions for CaesarEncoder
{
    fn encode(data: &Vec<u8>, key: Option<&RZ_KEY_TYPE>) -> std::io::Result<Vec<u8>>
    {
        let mut result: Vec<u8> = Vec::<u8>::with_capacity(data.len() + KEY_LENGTH_BITS);

        let (key_hashed, key_ref): ([u8; 32], &RZ_KEY_TYPE) = hash_key(key);

        result.extend_from_slice(&key_hashed);
        for &byte in data
        {
            result.push(byte.wrapping_add(*key_ref as u8));
        }
        return Ok(result);
    }

    fn decode(encoded_data: &Vec<u8>, possible_key: Option<&RZ_KEY_TYPE>) -> std::io::Result<Vec<u8>>
    { 
        if encoded_data.len() <= KEY_LENGTH_BITS { return Ok(Vec::new()); }

        let possible_key_ref: &RZ_KEY_TYPE = possible_key.expect("Error: No key");
        let hashed_key: &[u8] = &encoded_data[..32];
        
        if !verify_hash(possible_key_ref, hashed_key) { return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Invalid key")); }

        let mut result: Vec<u8> = Vec::with_capacity(encoded_data.len() - 32);
        for &byte in encoded_data.iter().skip(32) { result.push(byte.wrapping_sub(*possible_key_ref as u8)); }
        return Ok(result);
    }
}

pub struct CaesarEncoder
{
}