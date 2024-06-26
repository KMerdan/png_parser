// raw_png.rs
use crate::chunk::Chunk;
use crate::header::HeaderInfo;
use std::fmt;

use crate::chunk::ChunkType;
use crate::error::PngError;

#[derive(Debug)]
pub struct RawPng {
    pub signature: [u8; 8],
    pub header: HeaderInfo,
    pub chunks: Vec<Chunk>,
}

impl RawPng {
    pub fn new(
        signature: [u8; 8],
        header: HeaderInfo,
        chunks: Vec<Chunk>,
    ) -> Result<Self, PngError> {
        if !RawPng::is_signature_valid(signature) {
            return Err(PngError::InvalidSignature);
        }
        if !RawPng::verify_chunk_sequence(&chunks) {
            return Err(PngError::InvalidChunkSequence);
        }
        Ok(Self {
            signature,
            header,
            chunks,
        })
    }

    pub fn is_signature_valid(signature: [u8; 8]) -> bool {
        signature == [137, 80, 78, 71, 13, 10, 26, 10]
    }

    pub fn verify_chunk_sequence(chunks: &Vec<Chunk>) -> bool {
        // let mut has_ihdr = false; // we deal ihdr in other part
        let mut has_idat = false;
        let mut has_iend = false;

        for chunk in chunks {
            match chunk.chunk_type {
                // ChunkType::IHDR => has_ihdr = true,// we deal ihdr in other part
                ChunkType::IDAT => has_idat = true,
                ChunkType::IEND => has_iend = true,
                _ => {}
            }
        }

        has_idat && has_iend // && has_ihdr // we deal ihdr in other part
    }
}

impl fmt::Display for RawPng {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Raw PNG Information:\nSignature: {:?}\nHeader: {}\nNumber of Chunks: {}",
            self.signature,
            self.header.to_string(),
            self.chunks.len()
        )
    }
}
