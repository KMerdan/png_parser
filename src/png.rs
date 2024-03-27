// png.rs
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = !((crc & 1).wrapping_sub(1));
            crc = (crc >> 1) ^ (0xedb88320 & mask);
        }
    }
    !crc
}

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Debug)]
pub struct RawPng {
    pub header: Vec<u8>,
    pub chunks: Vec<Chunk>,
}

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    chunk_type: String,
    data: Vec<u8>,
    crc: u32,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Type: {} ({} bytes), it is {}",
            self.chunk_type,
            self.length,
            if self.is_valid() { "valid" } else { "invalid" }
        )
    }
}

impl Chunk {
    fn is_critical(&self) -> bool {
        let first_byte = self.chunk_type.as_bytes()[0];
        first_byte & 0x20 == 0
    }

    fn is_public(&self) -> bool {
        let second_byte = self.chunk_type.as_bytes()[1];
        second_byte & 0x20 == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        let third_byte = self.chunk_type.as_bytes()[2];
        third_byte & 0x20 == 0
    }

    fn verify_crc(&self) -> bool {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(self.chunk_type.as_bytes());
        buffer.extend_from_slice(&self.data);
        crc32(&buffer) == self.crc
    }

    fn is_valid(&self) -> bool {
        self.is_critical() && self.is_public() && self.is_reserved_bit_valid() && self.verify_crc()
        // self.verify_crc()
    }
}

pub struct PngReader {
    file_path: String,
}

impl PngReader {
    pub fn new(file_path: &str) -> io::Result<Self> {
        Ok(Self {
            file_path: file_path.to_string(),
        })
    }

    pub fn read_png(&mut self) -> io::Result<RawPng> {
        let buffer = self.read_file()?;
        self.validate_png_signature(&buffer)?;
        Ok(self.png_chunk_from_buffer(&buffer))
    }

    fn read_file(&self) -> io::Result<Vec<u8>> {
        let mut file = File::open(&self.file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn validate_png_signature(&self, buffer: &[u8]) -> io::Result<()> {
        let header = &buffer[0..8];
        if header != PNG_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid PNG header",
            ));
        }
        Ok(())
    }

    fn png_chunk_from_buffer(&self, buffer: &[u8]) -> RawPng {
        let header = &buffer[0..8];
        let mut offset = 8;
        let mut chunks = Vec::new();
        while offset < buffer.len() {
            let length = u32::from_be_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]);
            let chunk_type = String::from_utf8_lossy(&buffer[offset + 4..offset + 8]).to_string();
            let data = buffer[offset + 8..offset + 8 + length as usize].to_vec();
            let crc = u32::from_be_bytes([
                buffer[offset + 8 + length as usize],
                buffer[offset + 8 + length as usize + 1],
                buffer[offset + 8 + length as usize + 2],
                buffer[offset + 8 + length as usize + 3],
            ]);
            let chunk = Chunk {
                length,
                chunk_type,
                data,
                crc,
            };
            chunks.push(chunk);
            offset += 8 + length as usize + 4;
        }
        RawPng {
            header: header.to_vec(),
            chunks: chunks,
        }
    }
}
