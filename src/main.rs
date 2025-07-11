use std::io::{self, Error, ErrorKind};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;
use std::convert::TryFrom;

pub mod HuffmanObjects;
pub mod Codec;
pub mod EnvHandling;

use crate::HuffmanObjects::HuffmanEncoder;

use crate::Codec::CodecList;
use crate::Codec::CodecFunctions;

use crate::EnvHandling::write_decoded_file;
use crate::EnvHandling::write_encoded_file;
use crate::EnvHandling::read_file;
use crate::EnvHandling::validate_encoded_file;

pub type DetHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

fn encode_file(filepath: &str, codecs: &[u8]) -> io::Result<()> 
{
    match read_file(filepath)
    {
        Ok((mut global_buffer, original_len)) =>
        {
            for &codec_byte in codecs
            {
                let current_codec: CodecList = CodecList::try_from(codec_byte)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid codec found"))?;
                
                match current_codec
                {
                    CodecList::Huffman => 
                    {
                        match HuffmanEncoder::encode(&global_buffer)
                        {
                            Ok(huffman_encoded_data) => { global_buffer = huffman_encoded_data; }
                            Err(e) => 
                            {
                                eprintln!("An error occurred while encoding file with Huffman: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => { return Err(Error::new(ErrorKind::InvalidData, "Invalid codec found")); }
                }
            }

            match write_encoded_file(filepath, &global_buffer, &codecs)
            {
                Ok(_) => { return Ok(()); }
                Err(e) => 
                {
                    eprintln!("An error occurred while saving encoded file: {}", e);
                    return Err(e);
                }
            }
        }

        Err(e) => 
        {
            eprintln!("An error occurred while decoding file: {}", e);
            return Err(e);
        }
    }
}

fn decode_file(filepath: &str) -> std::io::Result<()>
{
    match read_file(filepath) 
    {
        Ok((mut global_buffer, original_len)) => 
        {
            match validate_encoded_file(global_buffer[0])
            {
                Ok(()) => {}
                Err(e) => 
                {
                    eprintln!("An error occurred while validating file: {}", e);
                    return Err(e);
                }
            }
            
            let mut current_byte: usize = 1;

            let codecs_len: usize = global_buffer[current_byte] as usize;
            current_byte += 1;

            let mut codecs: Vec<u8> = global_buffer[current_byte..current_byte+codecs_len].to_vec();
            codecs.reverse();

            current_byte += codecs_len;

            for codec_byte in codecs
            {
                let subbuffer = &global_buffer[current_byte..];

                let current_codec: CodecList = CodecList::try_from(codec_byte)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid codec found"))?;
                match current_codec 
                {
                    CodecList::Huffman => 
                    { 
                        match HuffmanEncoder::decode(&subbuffer.to_vec())
                        {
                            Ok(huffman_decoded_data) => 
                            { 
                                global_buffer = huffman_decoded_data;
                                current_byte = 0;
                            }
                            Err(e) => 
                            {
                                eprintln!("An error occurred while decoding file: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => { return Err(Error::new(ErrorKind::InvalidData, "Invalid codec found")); }
                }
            }
            
            match write_decoded_file(filepath, &global_buffer)
            {
                Ok(()) => 
                {
                    println!("File decoded and saved.");
                    return Ok(());
                }
                Err(e) => 
                {
                    eprintln!("Decoded content could not be saved due to an error: {}", e);
                    return Err(e);
                }
            }
        }

        Err(e) => 
        {
            eprintln!("An error occurred while decoding file: {}", e);
            return Err(e);
        }
    }
}

fn main() -> io::Result<()>
{
    match EnvHandling::check_entry()
    {
        Some((mode, filepath, codecs)) => 
        {   
            if mode == "-e" 
            {
                if let Some(codecs_vec) = codecs.as_ref() { encode_file(&filepath, codecs_vec); } 
                else 
                {
                    eprintln!("No codecs specified for encoding");
                    std::process::exit(1);
                }
            }
            else if mode == "-d" { decode_file(&filepath); }
            else { std::process::exit(1); }

            return Ok(());
        }
        None => { std::process::exit(1); }
    }
}