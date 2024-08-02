//! # Rust implementation of the DEFLATE algorithm
//! This project is a toy library written in Rust for decompressing data in the DEFLATE format.
//! It is primarily intended for educational purposes, facilitating a deeper understanding of the DEFLATE compression algorithm as outlined in RFC 1951.
//! By implementing this library, the goal is to gain practical experience with the intricacies of compression and decompression processes, as well as to enhance Rust programming skills.
//! It is not designed for production use but serves as a hands-on learning tool to explore the fundamentals of data compression.
pub mod bit_stream;

pub mod huffman;

pub mod inflate;
