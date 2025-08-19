use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, BlockDecrypt,
    KeyInit,
    generic_array::GenericArray,
};

use crate::Codec::CodecFunctions;
use crate::HashHandling::{verify_hash, hash_key};
use crate::{RZ_KEY_TYPE, KEY_LENGTH_BYTES, KEY_LENGTH_BITS};

impl CodecFunctions for AESEncoder
{
    fn encode(data: &Vec<u8>, key: Option<&RZ_KEY_TYPE>) -> std::io::Result<Vec<u8>>
    {
        // Generate validation hash (for key verification)
        let (validation_hash, key_ref): ([u8; 32], &RZ_KEY_TYPE) = hash_key(key);
        
        // Use user key directly as AES key (i128 = 16 bytes, perfect for AES-128)
        let aes_key: [u8; 16] = key_ref.to_be_bytes();
        
        // Create AES-128 cipher
        let cipher = Aes128::new_from_slice(&aes_key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to create cipher: {}", e)))?;
        
        // Prepare data for encryption (padding if necessary)
        let mut padded_data = data.clone();
        let block_size = 16;
        let padding_needed = block_size - (data.len() % block_size);
        if padding_needed < block_size 
        {
            padded_data.extend(std::iter::repeat(padding_needed as u8).take(padding_needed));
        } 
        else 
        {
            padded_data.extend(std::iter::repeat(block_size as u8).take(block_size));
        }
        
        // Encrypt data block by block
        let mut encrypted_data = Vec::with_capacity(padded_data.len());
        for chunk in padded_data.chunks(block_size) 
        {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            encrypted_data.extend_from_slice(&block);
        }
        
        // Create result: validation hash + encrypted data
        let mut result: Vec<u8> = Vec::with_capacity(32 + encrypted_data.len());
        result.extend_from_slice(&validation_hash);
        result.extend_from_slice(&encrypted_data);
        
        return Ok(result);
    }

    fn decode(encoded_data: &Vec<u8>, possible_key: Option<&RZ_KEY_TYPE>) -> std::io::Result<Vec<u8>>
    { 
        if encoded_data.len() < 33 
        { 
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Encoded data too short")); 
        }

        let validation_hash = &encoded_data[..32];
        let encrypted_data = &encoded_data[32..];

        let possible_key_ref: &RZ_KEY_TYPE = possible_key.expect("Error: No key");
        
        // Verify key using validation hash
        if !verify_hash(possible_key_ref, validation_hash) 
        { 
            return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Invalid key")); 
        }

        // Use user key directly as AES key (same process as encoding)
        let aes_key: [u8; 16] = possible_key_ref.to_be_bytes();
        
        // Create AES-128 cipher
        let cipher = Aes128::new_from_slice(&aes_key)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to create cipher: {}", e)))?;
        
        // Decrypt data block by block
        let mut decrypted_data = Vec::with_capacity(encrypted_data.len());
        for chunk in encrypted_data.chunks(16) 
        {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            decrypted_data.extend_from_slice(&block);
        }
        
        // Remove padding
        if let Some(&padding_size) = decrypted_data.last() 
        {
            if padding_size as usize <= decrypted_data.len() && padding_size <= 16 
            {
                decrypted_data.truncate(decrypted_data.len() - padding_size as usize);
            }
        }
        
        return Ok(decrypted_data);
    }
}

pub struct AESEncoder
{
}
