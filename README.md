# `inflate_toy`

`inflate_toy` is a toy Rust library designed for decompressing data in the DEFLATE format. This library is intended for educational purposes, helping users understand the DEFLATE compression algorithm as defined in RFC 1951. By implementing this crate, users can gain hands-on experience with the principles of compression and decompression while honing their Rust programming skills.

## Features

- **DEFLATE Decompression**: Decompresses data compressed using the DEFLATE algorithm.
- **Modular Design**: Organized into separate modules for bit stream handling, Huffman coding, and decompression.
- **Educational Tool**: Provides a practical implementation for learning about data compression.

## Installation

Add `inflate_toy` to your `Cargo.toml`:

```toml
[dependencies]
inflate_toy = "0.1"  # Replace with the latest version
```

## Usage

Here is a basic example of how to use the `inflate_toy` crate to decompress a byte array:

```rust
use inflate_toy::inflate::inflate_to_vec;

const DATA_COMPRESSED: &[u8] = &[
    // Your compressed data here
];

fn main() {
    let data = inflate_to_vec(DATA_COMPRESSED).unwrap();
    println!("{}", String::from_utf8_lossy(&data));
}
```

## Modules

- **`bit_stream`**: Handles reading and writing bits from/to a byte stream.
- **`huffman`**: Implements Huffman coding, including tables and symbol resolution.
- **`inflate`**: Contains the decompression logic for the DEFLATE algorithm.

## License

This crate is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Contributing

Contributions are welcome! Please open an issue or pull request to discuss changes or improvements.

## Acknowledgements

This crate is built for educational purposes and is inspired by the DEFLATE algorithm as specified in RFC 1951. It is not intended for production use.

---

Feel free to adjust or expand the sections as needed!