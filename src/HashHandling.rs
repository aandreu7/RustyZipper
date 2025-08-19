use sha2::{Sha256, Digest};
use sha2::digest::Output;

use crate::{RZ_KEY_TYPE, KEY_LENGTH_BYTES, KEY_LENGTH_BITS};

pub fn hash_key(key: Option<&RZ_KEY_TYPE>) -> ([u8; 32], &RZ_KEY_TYPE)
{
    let key_ref: &RZ_KEY_TYPE = key.expect("Error: No key");
    let key_bytes: [u8; 16] = key_ref.to_be_bytes();
    let key_hashed_raw: Output<Sha256> = Sha256::digest(&key_bytes);
    let mut key_hashed: [u8; 32] = [0u8; 32];
    key_hashed.copy_from_slice(key_hashed_raw.as_slice());
    return (key_hashed, key_ref);
}

pub fn verify_hash(possible_key: &RZ_KEY_TYPE, hashed_key: &[u8]) -> bool
{  
    let possible_key_bytes: [u8; 16] = possible_key.to_be_bytes();
    let possible_key_hashed: Output<Sha256> = Sha256::digest(&possible_key_bytes);
    return possible_key_hashed.as_slice() == hashed_key;
}   