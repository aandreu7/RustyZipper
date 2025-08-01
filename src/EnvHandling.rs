use std::env;
use std::fs::File;
use std::io::{self, Read, BufReader, Write, BufWriter, Error, ErrorKind};

use crate::Codec::CodecList;
use crate::DetHashMap;

pub fn write_decoded_file(filename: &str, decoded_data: &[u8]) -> std::io::Result<()>
{
    let full_path = format!("{}.decoded", filename);
    let mut file = File::create(full_path)?;
    file.write_all(decoded_data);
    return Ok(());
}

pub fn write_encoded_file(filename: &str, buffer: &[u8], codecs: &[u8]) -> std::io::Result<(String)> 
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
    return Ok((full_path));
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

pub fn check_entry() -> Option<(String, String, Option<Vec<u8>>, Option<Vec<i64>>)> 
{
    let args: Vec<String> = env::args().collect();

    if args.len() >= 3
    {
        let mode: &String = &args[1];
        let filepath: &String = &args[args.len()-1];

        match mode.as_str()
        {
            "-e" => 
            {
                let mut codecs: Vec<u8> = Vec::new();
                let mut keys: Vec<i64> = Vec::new();
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
                        let key: i64 = arg.parse().expect("Bad argument: expected number.");
                        keys.push(key)
                        key_needed = false;
                    }
                    match arg.as_str()
                    {
                        "--huffman" => { codecs.push(CodecList::Huffman as u8); }
                        "--rle" => { codecs.push(CodecList::RLE as u8); }
                        "--caesar" => 
                        { 
                            codecs.push(CodecList::Caesar as u8); }
                            key_needed = true;
                        _ =>
                        {
                            eprintln!("Incorrect codec: {}", arg);
                            return None;
                        }
                    }
                }
                return Some((mode.clone(), filepath.clone(), Some(codecs), Some(keys)));
            }
            "-d" => { if args.len() == 3 { return Some((mode.clone(), filepath.clone(), None)); } }
            _ => {}
        }
    }

    eprintln!("Incorrect use. Sintax: {} [-e [codecs]|-d] <path to file>", args[0]);
    return None;
}