use miniz_oxide;
use std::io::Result;

pub trait Inflate {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8>;
}

pub struct MyInflater;

impl Inflate for MyInflater {
    fn inflate_to_vec(&self, data: &[u8]) -> Vec<u8> {
        miniz_oxide::inflate::decompress_to_vec(data).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::read, io::Write, path::PathBuf};

    fn get_test_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
    }

    const DEFLATE_FILE: &str = "random_data.deflate";
    const DECOMPRESSED_FILE: &str = "random_data.decompressed";

    #[test]
    fn test_inflate() -> Result<()> {
        // Read the DEFLATE data
        let deflate_data = read(get_test_dir().join(DEFLATE_FILE))?;
        println!(
            "{:?}",
            &deflate_data
                .iter()
                .take(10)
                .map(|&b| format!("{:02x} ", b))
                .collect::<String>()
        );

        // Decompress the DEFLATE data using your custom inflater
        let inflater = MyInflater;
        let decompressed_data = inflater.inflate_to_vec(&deflate_data);

        // Print the first 10 bytes of the decompressed data in hexadecimal format
        println!(
            "{} {:?}",
            decompressed_data.len(),
            &decompressed_data
                .iter()
                .take(10)
                .map(|&b| format!("{:02x} ", b))
                .collect::<String>()
        );

        // Write the decompressed data to a file for comparison
        let mut file = std::fs::File::create(get_test_dir().join(DECOMPRESSED_FILE))?;
        file.write(&decompressed_data)?;

        Ok(())
    }
}
