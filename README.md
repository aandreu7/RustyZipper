# RustyZipper

A powerful command-line file compression and encryption tool written in Rust. RustyZipper allows you to compress and encrypt any file type using multiple algorithms in any order you specify. It supports classic compression algorithms like Huffman and RLE, as well as encryption methods like Caesar cipher and AES-128.

## Features

- **Multiple Compression Algorithms**: Huffman encoding, Run-Length Encoding (RLE)
- **Multiple Encryption Methods**: Caesar cipher, AES-128 encryption
- **Flexible Pipeline**: Apply multiple algorithms in any order
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Fast & Efficient**: Written in Rust for optimal performance
- **Secure**: Uses cryptographic-grade algorithms with proper key validation

## Installation

### Prerequisites
- Rust (latest stable version)
- Cargo

### Build from Source
```bash
git clone https://github.com/aandreu7/RustyZipper.git
cd RustyZipper
cargo build --release
```

## Usage

### Basic Syntax
```bash
# Encode/Compress/Encrypt
RustyZipper -e [ codec/cypher (cypher key) ] <file_path>

# Decode/Decompress/Decrypt  
RustyZipper -d [ (cypher key) ] <file_path>
```

Cypher keys must be specified in the same order for encrypting and decrypting, although these operations are inverse. See examples for a further understanding of this.

### Examples

#### Simple Compression
```bash
# Compress with Huffman
RustyZipper -e --huffman document.txt

# Compress with RLE
RustyZipper -e --rle image.png
```

#### Encryption Only
```bash
# Encrypt with Caesar cipher (key: 12345)
RustyZipper -e --caesar 12345 secret.txt

# Encrypt with AES-128 (key: 98765)
RustyZipper -e --aes 98765 confidential.pdf
```

#### Combined Operations
```bash
# Compress with Huffman, then encrypt with AES
RustyZipper -e --huffman --aes 12345 large_file.dat

# Encrypt with Caesar, then compress with RLE
RustyZipper -e --caesar 42 --rle data.bin
```

#### Decryption/Decompression
```bash
# Decrypt AES-encrypted file
RustyZipper -d 12345 file.rsz

# Decrypt Caesar, then decompress RLE
RustyZipper -d 42 file.rsz
```

### Keys order in multiple encryptions
```bash
# The encryption order is processed from left to right as specified (firstly, encrypting with caesar and then using aes)
RustyZipper -e --caesar 12345 --aes 6789 file.txt

# For decryption, keys are specified following the same order of encryption
RustyZipper -d 12345 6789 file.txt.rsz

# RustyZipper will decrypt using keys from right to left, so 6789 will be used to decrypt using aes, and 12345 for caesar
```

## Supported Algorithms

### Compression Codecs
- **Huffman**: Variable-length encoding for optimal compression
- **RLE**: Run-Length Encoding for repetitive data

### Encryption Methods
- **Caesar Cipher**: Simple substitution cipher with key validation
- **AES-128**: Advanced Encryption Standard with SHA-256 key derivation

## File Format

RustyZipper creates `.rsz` files that contain:
- File signature for validation
- Algorithm pipeline information
- Encoded/encrypted data
- Hashed keys using SHA-256 algorithm

## Security Features

- **Key Validation**: All encrypted files include hash validation
- **AES-128**: Industry-standard encryption with proper padding
- **SHA-256**: Cryptographic hash functions for key derivation
- **Error Handling**: Comprehensive error checking and validation

## Architecture

## Contributing

Any contribution is **more than welcome**. You can add your own encryption or compression method if it is not already included.

All contribution requests will be **thoroughly reviewed to ensure the highest security standards**.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
