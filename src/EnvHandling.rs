use std::env;
use std::fs::File;
use std::fs;
use std::io::{Read, BufReader, Write, BufWriter, Error, ErrorKind};

use crate::Codec::CodecList;
use crate::{DetHashMap, RZ_KEY_TYPE, KEY_LENGTH_BYTES, KEY_LENGTH_BITS};

pub fn write_decoded_file(filename: &str, decoded_data: &[u8]) -> std::io::Result<()>
{
    // Remove .rsz extension to restore original filename
    let original_filename = if filename.ends_with(".rsz") { &filename[..filename.len() - 4] } 
    else { filename };
    
    let mut file = File::create(original_filename)?;
    file.write_all(decoded_data);

    // Remove .rsz file
    if filename.ends_with(".rsz") { fs::remove_file(filename)?; }

    return Ok(());
}

pub fn write_encoded_file(filename: &str, buffer: &[u8], codecs: &[u8]) -> std::io::Result<String> 
{
    let full_path = format!("{}.rsz", filename);
    let mut file = BufWriter::new(File::create(&full_path)?);

    // Writes metadata
    let offset = 1 + codecs.len();
    let mut full_buffer = Vec::with_capacity(offset + buffer.len());

    // Writes RustyZipper signature
    let rszSignature: u8 = CodecList::RustyZipper as u8;
    full_buffer.push(rszSignature);

    // Writes number of codecs used and codecs ids
    full_buffer.push(codecs.len() as u8);
    full_buffer.extend_from_slice(codecs);

    // Writes encoded data
    full_buffer.extend_from_slice(buffer);

    file.write_all(&full_buffer)?;
    return Ok(full_path);
}

pub fn read_file(filename: &str) -> std::io::Result<(Vec<u8>, usize)>
{
    let mut file = BufReader::new(File::open(filename)?);
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;
    let len = buffer.len();
    return Ok((buffer, len));
}

pub fn validate_encoded_file(first_byte: u8) -> std::io::Result<()>
{
    // Verifies RustyZipper signature
    if first_byte != CodecList::RustyZipper as u8 { return Err(Error::new(ErrorKind::InvalidData, "File has not been encoded using RustyZipper")); }
    return Ok(());
}

pub fn check_entry() -> Option<(String, String, Option<Vec<u8>>, Option<Vec<RZ_KEY_TYPE>>)> 
{
    let args: Vec<String> = env::args().collect();

    if args.len() >= 3
    {
        let mode: &String = &args[1];
        let filepath: &String = &args[args.len()-1];

        let mut keys: Vec<RZ_KEY_TYPE> = Vec::new();

        match mode.as_str()
        {
            "-e" => 
            {
                let mut codecs: Vec<u8> = Vec::new();
                let mut key_needed: bool = false;
                if args.len() == 3
                {
                    eprintln!("Incorrect use. Indicate desired codecs after -e");
                    return None;
                } 
                for arg in &args[2..args.len()-1]
                {
                    if key_needed
                    {
                        // Take up to 16 bytes and pack them into an i128 (big-endian)
                        let key_bytes = arg.as_bytes();
                        let mut arr = [0u8; 16];
                        let n = key_bytes.len().min(16);
                        arr[16 - n..].copy_from_slice(&key_bytes[..n]);
                        let key: RZ_KEY_TYPE = i128::from_be_bytes(arr);
                        keys.push(key);
                        key_needed = false;
                        continue;
                    }
                    match arg.as_str()
                    {
                        "--huffman" => { codecs.push(CodecList::Huffman as u8); }
                        "--rle" => { codecs.push(CodecList::RLE as u8); }
                        "--caesar" => 
                        { 
                            codecs.push(CodecList::Caesar as u8);
                            key_needed = true;
                        }
                        "--aes" => 
                        { 
                            codecs.push(CodecList::AES as u8);
                            key_needed = true;
                        }
                        _ =>
                        {
                            eprintln!("Incorrect codec: {}", arg);
                            return None;
                        }
                    }
                }
                return Some((mode.clone(), filepath.clone(), Some(codecs), Some(keys)));
            }
            "-d" => 
            { 
                for arg in &args[2..args.len()-1]
                {
                    // Take up to 16 bytes and pack them into an i128 (big-endian)
                    let key_bytes = arg.as_bytes();
                    let mut arr = [0u8; 16];
                    let n = key_bytes.len().min(16);
                    arr[16 - n..].copy_from_slice(&key_bytes[..n]);
                    let key: RZ_KEY_TYPE = i128::from_be_bytes(arr);
                    keys.push(key);
                }
                return Some((mode.clone(), filepath.clone(), None, Some(keys)));
            }
            _ => {}
        }
    }

    eprintln!("Incorrect use. Sintax: {} [-e [codecs]|-d] <path to file>", args[0]);
    return None;
}