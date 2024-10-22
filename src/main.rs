use inflate_toy::inflate::inflate_to_vec;

const DATA_COMPRESSED: &[u8] = &[
    0xed, 0x90, 0xcb, 0x51, 0x04, 0x31, 0x0c, 0x44, 0xef, 0x44, 0xd1, 0x01, 0x6c, 0x11, 0x05, 0xc7,
    0xbd, 0x12, 0x80, 0xb0, 0xc5, 0xd0, 0x55, 0xb6, 0x3c, 0x6b, 0x4b, 0x5b, 0x84, 0x8f, 0x66, 0xa6,
    0x08, 0x81, 0x1b, 0x3e, 0xc9, 0x87, 0x7e, 0xfd, 0xb9, 0x8f, 0xa9, 0x1d, 0xdc, 0x57, 0x74, 0xd4,
    0xd1, 0xc6, 0xc4, 0xa2, 0x43, 0xba, 0xfa, 0x0d, 0x65, 0xd8, 0xd2, 0xe2, 0xea, 0x31, 0x21, 0x95,
    0x3b, 0x57, 0xa1, 0x6d, 0xd0, 0x46, 0x7f, 0x41, 0xbe, 0xa5, 0x35, 0x35, 0x50, 0xc6, 0xea, 0xa3,
    0xc2, 0xb5, 0xef, 0xa9, 0xa7, 0x15, 0x56, 0xd6, 0x30, 0x47, 0x38, 0x9a, 0x7c, 0xa4, 0x03, 0xd4,
    0x2f, 0xba, 0xa2, 0xcb, 0x66, 0x02, 0x69, 0x7c, 0x84, 0x9c, 0x94, 0x77, 0x87, 0x1a, 0x7b, 0x3a,
    0xa0, 0xf3, 0x38, 0x9e, 0xf9, 0x95, 0x7e, 0xc3, 0x23, 0xb8, 0x60, 0x63, 0xf9, 0x8c, 0x0a, 0xfd,
    0xd6, 0x59, 0xe8, 0xe2, 0x1c, 0x86, 0x68, 0x4d, 0x7a, 0x19, 0x17, 0x9c, 0xeb, 0xc4, 0x18, 0x17,
    0x0f, 0xc3, 0x93, 0xcc, 0x3d, 0x05, 0x50, 0xc9, 0x0a, 0x3d, 0xa3, 0x8d, 0xab, 0x4a, 0x3a, 0xfa,
    0x2b, 0xde, 0x0e, 0xac, 0x84, 0x2b, 0x38, 0x23, 0x03, 0x5d, 0xad, 0x69, 0x27, 0x65, 0xea, 0x3e,
    0xf5, 0x4b, 0xad, 0xea, 0xcc, 0x19, 0x68, 0x78, 0x8e, 0x16, 0x7b, 0xda, 0x6a, 0xc6, 0xca, 0xde,
    0xd0, 0xb5, 0x14, 0x85, 0xad, 0xfd, 0xee, 0x95, 0xdd, 0x02, 0x9f, 0xb1, 0x51, 0x1c, 0x76, 0x04,
    0x3b, 0x39, 0xf7, 0xff, 0x59, 0xff, 0x62, 0xd6, 0x1f,
];

fn main() {
    let data = inflate_to_vec(DATA_COMPRESSED).unwrap();
    println!("{}", String::from_utf8_lossy(&data));
    println!("Data: \n{}", display_data(&data));
}


/// Display the data in hex format.
fn display_data(data: &[u8]) -> String {
    let mut result = String::new();

    for (i, chunk) in data.chunks(16).enumerate() {
        // Print the offset
        result.push_str(&format!("{:08x}: ", i * 16));

        // Print the byte values in hex
        for byte in chunk {
            result.push_str(&format!("{:02x} ", byte));
        }

        // If the chunk is less than 16 bytes, fill the gap
        for _ in 0..(16 - chunk.len()) {
            result.push_str("   ");
        }

        // Print the ASCII representation
        result.push_str(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() {
                result.push(*byte as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
    }

    result
}
