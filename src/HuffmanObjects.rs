use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::fmt;
use std::io::{Cursor, Write, Read, Result, Seek, SeekFrom};

use crate::Codec::CodecFunctions;
use crate::DetHashMap;

fn invert_codes(codes: &DetHashMap<u8, Vec<bool>>) -> DetHashMap<Vec<bool>, u8>
{
    let mut inverted: DetHashMap<Vec<bool>, u8> = DetHashMap::default();
    for (&byte, code_bits) in codes.iter() { inverted.insert(code_bits.clone(), byte); }
    return inverted;
}

fn decode_data_direct(codes: &DetHashMap<u8, Vec<bool>>, encoded_data: &[u8], original_len: usize) -> Vec<u8>
{
    let inverted: DetHashMap<Vec<bool>, u8> = invert_codes(codes);
    let bits = bits_from_bytes(encoded_data);
    let mut result: Vec<u8> = Vec::with_capacity(original_len);
    let mut buffer: Vec<bool> = Vec::new();

    for bit in bits
    {
        buffer.push(bit);
        if let Some(&byte) = inverted.get(&buffer)
        {
            result.push(byte);
            buffer.clear();
            if result.len() == original_len { break; }
        }
    }

    return result;
}

fn bits_from_bytes(bytes_list: &[u8]) -> Vec<bool> 
{
    let mut bits: Vec<bool> = Vec::new();
    for byte in bytes_list 
    {
        // If current bit from current byte is 0, adds False. Else, adds True.
        for i in (0..8).rev() { bits.push((byte >> i) & 1 == 1); }
    }
    return bits;
}

fn bytes_from_bits(bits_list: &[bool]) -> Vec<u8>
{
    let mut bytes: Vec<u8> = Vec::new();
    let mut current_byte: u8 = 0;
    let mut bits_in_current_byte = 0;

    for bit in bits_list 
    {
        current_byte <<= 1;            // Left shift to add new bit
        if *bit { current_byte |= 1; } // If bit is 1, puts it on least significant position. Else, let it as zero
        bits_in_current_byte += 1;

        if bits_in_current_byte == 8 
        {
            bytes.push(current_byte);
            current_byte = 0;
            bits_in_current_byte = 0;
        }
    }

    // If bits are left (less than 8) in the last byte, stuffs last byte with zeros
    if bits_in_current_byte > 0 
    {
        current_byte <<= 8 - bits_in_current_byte;
        bytes.push(current_byte);
    }

    return bytes;
}

#[derive(PartialEq, Eq)]
pub enum HuffmanNode
{
    Leaf { byte: u8, freq: usize },
    Internal { internalFreq: usize, left: Box<HuffmanNode>, right: Box<HuffmanNode> },
}

impl fmt::Display for HuffmanNode
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        fn fmt_rec(node: &HuffmanNode, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result
        {
            let indent = " ".repeat(depth);
            match node
            {
                HuffmanNode::Leaf { byte, freq } => { return writeln!(f, "{}Leaf(byte: {:08b}, freq: {})", indent, byte, freq); }
                HuffmanNode::Internal { internalFreq, left, right } =>
                {
                    writeln!(f, "{}Internal(freq: {})", indent, internalFreq)?;
                    fmt_rec(left, f, depth + 1)?;
                    return fmt_rec(right, f, depth + 1);
                }
            }
        }
        return fmt_rec(self, f, 0);
    }
}

#[derive(PartialEq, Eq)]
struct HuffmanTreeItem(pub usize, pub Box<HuffmanNode>);

impl Ord for HuffmanTreeItem
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering 
    { 
        other.0.cmp(&self.0) 
    }
}

impl PartialOrd for HuffmanTreeItem
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl HuffmanEncoder
{
    pub fn obtain_frequencies(vocabulary: &Vec<u8>) ->  DetHashMap<u8, usize>
    {
        let mut frequencies: DetHashMap<u8, usize> = DetHashMap::default();

        for &word in vocabulary { *frequencies.entry(word).or_insert(0) += 1; }

        return frequencies;
    }

    pub fn new(frequencies: &DetHashMap<u8, usize>) -> Self 
    {
        let mut tree: BinaryHeap<HuffmanTreeItem> = BinaryHeap::<HuffmanTreeItem>::new();
        let mut nodes: Vec<Box<HuffmanNode>> = Vec::<Box<HuffmanNode>>::new();

        // Step 1: Adds leaf nodes into the heap
        for (&byte, &freq) in frequencies.iter() 
        {
            let leaf = Box::new(HuffmanNode::Leaf { byte, freq });
            tree.push(HuffmanTreeItem(freq, leaf));
        }

        // Step 2: Builds tree combining nodes
        while tree.len() > 1 
        {
            // Extracts nodes with lowest frequencies
            let HuffmanTreeItem(freq1, left) = tree.pop().unwrap();
            let HuffmanTreeItem(freq2, right) = tree.pop().unwrap();

            // Creates intern node with the sum of frequencies
            let internal_freq = freq1 + freq2;
            let internal_node = Box::new
            (
                HuffmanNode::Internal 
                {
                    internalFreq: internal_freq,
                    left,
                    right,
                }
            );

            // Inserts new intern node
            tree.push(HuffmanTreeItem(internal_freq, internal_node));
        }
        
        let HuffmanTreeItem(_freq, root_node) = tree.pop().unwrap();
        let root: Option<Box<HuffmanNode>> = Some(root_node);
        
        return HuffmanEncoder { root, tree, nodes, };
    }

    fn generate_codes(&self) -> DetHashMap<u8, Vec<bool>> 
    {
        let mut codes = DetHashMap::default();

        fn traverse(node: &HuffmanNode, prefix: Vec<bool>, codes: &mut DetHashMap<u8, Vec<bool>>) 
        {
            match node 
            {
                HuffmanNode::Leaf { byte, .. } => { codes.insert(*byte, prefix); },
                HuffmanNode::Internal { left, right, .. } => 
                {
                    let mut left_prefix = prefix.clone();
                    left_prefix.push(false);
                    traverse(left, left_prefix, codes);

                    let mut right_prefix = prefix;
                    right_prefix.push(true);
                    traverse(right, right_prefix, codes);
                }
            }
        }

        if let Some(root) = &self.root { traverse(root, Vec::new(), &mut codes); }
        else { eprintln!("Error: root does not exist, so codes cannot be generated."); }

        return codes;
    }

    pub fn encode_data(&self, vocabulary: &Vec<u8>) -> (DetHashMap<u8, Vec<bool>>, Vec<u8>) 
    {
        let codes: DetHashMap<u8, Vec<bool>> = self.generate_codes();
        let mut bit_buffer: Vec<bool> = Vec::new();

        // 1. For each file's byte, overwrittes the original byte with its corresponding code (sequence of bits, Variable Length Coding)
        for &byte in vocabulary 
        {
            // 2. Adds code (sequence of bits) from byte to buffer
            if let Some(code) = codes.get(&byte) { bit_buffer.extend(code); } 
            else { panic!("Byte with no Huffman code: {}", byte); }
        }

        // 3. Converts sequence of bits to real bytes (u8):
        // It is not possible to store sequences of bits in a file, they have to be converted into bytes (u8) previously
        let encoded_bytes: Vec<u8> = bytes_from_bits(&bit_buffer);

        return (codes, encoded_bytes);
    }

    pub fn write_to_buffer(codes: &DetHashMap<u8, Vec<bool>>, encoded_data: &[u8], original_len: usize) -> std::io::Result<Vec<u8>>
    {
        let mut cursor = Cursor::new(Vec::new());

        // 1. Writes number of codes (u16)
        let codes_count = codes.len() as u16;
        cursor.write_all(&codes_count.to_be_bytes())?;

        // 2. Writes, for each code:
        //    - 1 byte: character
        //    - 1 byte: length in bits
        //    - N bytes: character bit-coded
        for (&byte, code_bits) in codes.iter()
        {
            cursor.write_all(&[byte])?; // character

            let code_len = code_bits.len() as u8;
            cursor.write_all(&[code_len])?; // length in bits

            let mut byte_buffer = 0u8;
            let mut bits_in_buffer = 0;

            for &bit in code_bits
            {
                byte_buffer <<= 1;
                if bit { byte_buffer |= 1; }
                bits_in_buffer += 1;
                if bits_in_buffer == 8
                {
                    cursor.write_all(&[byte_buffer])?; // character bit-coded
                    byte_buffer = 0;
                    bits_in_buffer = 0;
                }
            }

            // If bits are left to complete a byte, stuff it whit 0s at most-significant positions
            if bits_in_buffer > 0
            {
                byte_buffer <<= 8 - bits_in_buffer;
                cursor.write_all(&[byte_buffer])?; // character bit-coded
            }
        }

        // 3. Writes original length (u32)
        cursor.write_all(&(original_len as u32).to_be_bytes())?;

        // 4. Writes codified data
        cursor.write_all(encoded_data)?;

        return Ok(cursor.into_inner());
    }

    pub fn read_from_buffer(buffer: &[u8]) -> std::io::Result<(DetHashMap<u8, Vec<bool>>, Vec<u8>, usize)>
    {
        let mut cursor = Cursor::new(buffer);
        
        // 1. Reads number of codes (u16)
        let mut buffer2 = [0u8; 2];
        cursor.read_exact(&mut buffer2)?; // Reads two bytes
        let codes_count = u16::from_be_bytes(buffer2);

        // 2. Reads, for each code:
        //    - 1 byte: character
        //    - 1 byte: length in bits
        //    - N bytes: character bit-coded
        let mut codes: DetHashMap<u8, Vec<bool>> = DetHashMap::default();
        for _ in 0..codes_count
        {
            let mut byte_buf = [0u8; 1];
            cursor.read_exact(&mut byte_buf)?; // character
            let byte: u8 = byte_buf[0];

            let mut len_buf = [0u8; 1];
            cursor.read_exact(&mut len_buf)?; // length in bits
            let code_len = len_buf[0] as usize;

            // It reads bytes, but works with bits, so conversion needed
            let bytes_needed = (code_len + 7) / 8;
            let mut code_bytes = vec![0u8; bytes_needed];
            cursor.read_exact(&mut code_bytes)?; // characters bit-coded

            // Converts from bytes to bits (only code_len bits)
            let mut code_bits: Vec<bool> = Vec::with_capacity(code_len);
            for i in 0..code_len
            {
                let byte_index = i / 8;
                let bit_index = 7 - (i % 8);
                let bit = (code_bytes[byte_index] >> bit_index) & 1 == 1;   // Locates searched bit at byte's least-significant position, 
                                                                            // so as to get its value
                code_bits.push(bit);
            }

            codes.insert(byte, code_bits);
        }

        // 3. Reads original length (u32)
        let mut len_buf4 = [0u8; 4]; // 4 bytes needed to read a u32 number
        cursor.read_exact(&mut len_buf4)?;
        let original_len = u32::from_be_bytes(len_buf4) as usize;

        // 4. Reads codified data
        let mut encoded_data: Vec<u8> = Vec::new();
        cursor.read_to_end(&mut encoded_data)?;

        return Ok((codes, encoded_data, original_len));
    }

    pub fn decode_data(&self, encoded: &Vec<u8>, original_len: usize) -> Vec<u8>
    {
        let bits: Vec<bool> = bits_from_bytes(encoded);
        let mut result = Vec::with_capacity(original_len);

        // 1. Ensures a root node exists, and gets its reference
        let mut node = self.root.as_ref().expect("No root node");
        let mut current = node;

        // 2. Gets codified bytes
        for bit in bits
        {
            match &**current
            {
                HuffmanNode::Leaf { byte, .. } =>
                {
                    result.push(*byte);
                    if result.len() == original_len { break; }
                    current = node;
                }
                HuffmanNode::Internal { left, right, .. } => { current = if bit { right } else { left }; }
            }
        }

        if let HuffmanNode::Leaf { byte, .. } = &**current
        {
            // In case it finished in a leaf, pushes last byte
            if result.len() < original_len { result.push(*byte); }
        }

        return result;
    }

    pub fn print(&self)
    {
        if let Some(node) = &self.root { println!("{}", node); }
        else { println!("Empty tree."); }
    }
}

impl CodecFunctions for HuffmanEncoder
{
    fn encode(data: &Vec<u8>) -> std::io::Result<Vec<u8>>
    {
        let original_len = data.len();
        let frequencies: DetHashMap<u8, usize> = Self::obtain_frequencies(&data);
        let mut freqTree: Self = Self::new(&frequencies);
        let (codes, encoded_data) = freqTree.encode_data(&data);

        return Self::write_to_buffer(&codes, &encoded_data, original_len);
    }

    fn decode(encoded_data: &Vec<u8>) -> std::io::Result<Vec<u8>>
    {
        match Self::read_from_buffer(encoded_data)
        {
            Ok((codes, huffman_encoded_data, original_len)) => { return Ok(decode_data_direct(&codes, &huffman_encoded_data, original_len)); }
            Err(e) => 
            {
                eprintln!("An error occurred while decoding with Huffman: {}", e);
                return Err(e);
            }
        }
    }
}

pub struct HuffmanEncoder
{
    root: Option<Box<HuffmanNode>>,
    tree: BinaryHeap<HuffmanTreeItem>,
    nodes: Vec<Box<HuffmanNode>>,
}