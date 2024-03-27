// chunk.rs
use std::fmt;

const IHDR: [u8; 4] = [73, 72, 68, 82];
const IDAT: [u8; 4] = [73, 68, 65, 84];
const IEND: [u8; 4] = [73, 69, 78, 68];

#[derive(Debug)]
pub enum ChunkType {
    IHDR,
    IDAT,
    IEND,
    Unknown,
}

impl ChunkType {
    pub fn from_str(chunk_type: &str) -> Self {
        match chunk_type {
            "IHDR" => ChunkType::IHDR,
            "IDAT" => ChunkType::IDAT,
            "IEND" => ChunkType::IEND,
            _ => ChunkType::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ChunkType::IHDR => "IHDR",
            ChunkType::IDAT => "IDAT",
            ChunkType::IEND => "IEND",
            ChunkType::Unknown => "unknown",
        }
    }

    pub fn is_same_as(&self, chunk_type: &ChunkType) -> bool {
        self.as_bytes() == chunk_type.as_bytes()
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            ChunkType::IHDR => IHDR,
            ChunkType::IDAT => IDAT,
            ChunkType::IEND => IEND,
            ChunkType::Unknown => [0, 0, 0, 0],
        }
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        match bytes {
            IHDR => ChunkType::IHDR,
            IDAT => ChunkType::IDAT,
            IEND => ChunkType::IEND,
            _ => ChunkType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub length: u32,
    pub chunk_type: ChunkType,
    pub data: Vec<u8>,
    pub crc: u32,
}

impl Chunk {
    pub fn new(length: u32, chunk_type: ChunkType, data: Vec<u8>, crc: u32) -> Self {
        Self {
            length,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn is_critical(&self) -> bool {
        self.chunk_type.as_bytes()[0].is_ascii_uppercase()
    }

    pub fn is_public(&self) -> bool {
        self.chunk_type.as_bytes()[1].is_ascii_uppercase()
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        self.chunk_type.as_bytes()[2].is_ascii_uppercase()
    }

    pub fn verify_crc(&self) -> bool {
        crc32(&self.chunk_type.as_bytes(), &self.data) == self.crc
    }

    pub fn is_valid(&self) -> bool {
        self.is_critical() && self.is_public() && self.is_reserved_bit_valid() && self.verify_crc()
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Type: {} ({} bytes), CRC: {}, Validity: {}",
            self.chunk_type.as_str(),
            self.length,
            self.crc,
            if self.is_valid() { "Valid" } else { "Invalid" }
        )
    }
}

fn crc32(chunk_type: &[u8], data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in chunk_type.iter().chain(data.iter()) {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
