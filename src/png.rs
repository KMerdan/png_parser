// png.rs
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use flate2::read::ZlibDecoder;

use crate::chunk::{Chunk, ChunkType};
use crate::header::HeaderInfo;
use crate::image_type::{Brightness, VisualData, PNG};
use crate::raw_data::RawPng;

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

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
        self.png_chunk_from_buffer(&buffer)
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

    fn png_chunk_from_buffer(&self, buffer: &[u8]) -> io::Result<RawPng> {
        let header_slice = &buffer[0..8];
        let mut offset = 8;
        let mut chunks = Vec::new();

        // Read the IHDR chunk
        let ihdr_length = u32::from_be_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as usize;
        let ihdr_data = &buffer[offset + 8..offset + 8 + ihdr_length];
        let header_info = HeaderInfo::new(ihdr_data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid IHDR chunk"))?;
        offset += 8 + ihdr_length + 4;

        // Read the remaining chunks
        while offset < buffer.len() {
            let length = u32::from_be_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as usize;
            let chunk_type = ChunkType::from_bytes([
                buffer[offset + 4],
                buffer[offset + 5],
                buffer[offset + 6],
                buffer[offset + 7],
            ]);
            // let chunk_type = String::from_utf8_lossy(&buffer[offset + 4..offset + 8]).to_string();
            let data = buffer[offset + 8..offset + 8 + length].to_vec();
            let crc = u32::from_be_bytes([
                buffer[offset + 8 + length],
                buffer[offset + 8 + length + 1],
                buffer[offset + 8 + length + 2],
                buffer[offset + 8 + length + 3],
            ]);
            let chunk = Chunk {
                length: length as u32,
                chunk_type,
                data,
                crc,
            };
            chunks.push(chunk);
            offset += 8 + length + 4;
        }
        let header: [u8; 8] = header_slice.try_into().unwrap();
        let raw_png = RawPng::new(header, header_info, chunks);
        Ok(raw_png)
    }

    pub fn to_brightness_data(
        &self,
        raw_png: &RawPng,
        step_size: usize,
    ) -> io::Result<Option<VisualData>> {
        let required_chunk_type = ChunkType::IDAT;
        let mut idat_data = Vec::new();
        for chunk in &raw_png.chunks {
            if chunk.chunk_type.is_same_as(&required_chunk_type) && chunk.is_valid() {
                idat_data.extend_from_slice(&chunk.data);
            }
        }

        let decompressed_data = PngReader::decompress_data(&idat_data)?;
        let unfiltered_data = PngReader::unfilter_data(&decompressed_data, &raw_png.header)?;
        let visual_data = PngReader::convert_to_visual_code(&unfiltered_data, &raw_png.header)?;
        let visual_data_result = PngReader::reshape_data(&visual_data, &raw_png.header, step_size)?;

        let brightness_data =
            PngReader::brightness_representation(visual_data_result.unwrap(), &raw_png.header)?;
        Ok(brightness_data)
    }

    fn decompress_data(data: &[u8]) -> io::Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;
        Ok(decompressed_data)
    }

    fn unfilter_data(data: &[u8], header: &HeaderInfo) -> io::Result<Vec<u8>> {
        let bytes_per_pixel = match header.color_type {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid color type",
                ))
            }
        };
        let scanline_length = (header.width * bytes_per_pixel as u32) as usize;
        let mut unfiltered_data = Vec::new();
        // let mut previous_scanline = vec![0; scanline_length];
        let mut offset = 0;
        while offset < data.len() {
            let filter_type = data[offset];
            let scanline = &data[offset + 1..offset + 1 + scanline_length];
            let unfiltered_scanline = match filter_type {
                0 => PngReader::unfilter_none(scanline)?,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid filter type or not implemented yet",
                    ))
                }
            };
            unfiltered_data.extend_from_slice(&unfiltered_scanline);
            // previous_scanline = unfiltered_scanline;
            offset += 1 + scanline_length;
        }
        Ok(unfiltered_data)
    }

    fn convert_to_visual_code(data: &[u8], header: &HeaderInfo) -> io::Result<Vec<u8>> {
        let bytes_per_pixel = match header.color_type {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid color type",
                ))
            }
        };
        let mut rgb_data = Vec::new();
        let mut offset = 0;
        while offset < data.len() {
            let pixel = match header.color_type {
                0 => {
                    let gray = data[offset];
                    vec![gray, gray, gray, 255]
                }
                2 => {
                    let r = data[offset];
                    let g = data[offset + 1];
                    let b = data[offset + 2];
                    vec![r, g, b, 255]
                }
                3 => {
                    let palette_index = data[offset];
                    let palette_offset = palette_index as usize * 3;
                    let r = data[palette_offset];
                    let g = data[palette_offset + 1];
                    let b = data[palette_offset + 2];
                    vec![r, g, b, 255]
                }
                4 => {
                    let gray = data[offset];
                    let alpha = data[offset + 1];
                    vec![gray, gray, gray, alpha]
                }
                6 => {
                    let r = data[offset];
                    let g = data[offset + 1];
                    let b = data[offset + 2];
                    let alpha = data[offset + 3];
                    vec![r, g, b, alpha]
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid color type",
                    ))
                }
            };
            rgb_data.extend_from_slice(&pixel);
            offset += bytes_per_pixel;
        }
        Ok(rgb_data)
    }

    fn unfilter_none(scanline: &[u8]) -> io::Result<Vec<u8>> {
        Ok(scanline.to_vec())
    }
    fn reshape_data(
        data: &[u8],
        header: &HeaderInfo,
        step_size: usize,
    ) -> io::Result<Option<VisualData>> {
        let bytes_per_pixel = match header.color_type {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid color type",
                ))
            }
        };
        let mut reshaped_data = Vec::new();
        let width = header.width as usize;
        let height = header.height as usize;
        for y in (0..height).step_by(step_size) {
            for x in (0..width).step_by(step_size) {
                let offset = (y * width + x) * bytes_per_pixel;
                if offset + bytes_per_pixel <= data.len() {
                    reshaped_data.extend_from_slice(&data[offset..offset + bytes_per_pixel]);
                }
            }
        }
        let image = PNG {
            shape: (
                header.width / step_size as u32,
                header.height / step_size as u32,
            ),
            data: reshaped_data,
        };
        Ok(Some(VisualData::RGBA(image)))
    }

    fn brightness_representation(
        b_data: VisualData,
        header: &HeaderInfo,
    ) -> io::Result<Option<VisualData>> {
        let bytes_per_pixel = match header.color_type {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid color type",
                ));
            }
        };
        let data: Vec<u8> = (&b_data).into();
        let mut _brightness_data = Vec::new();
        let mut offset = 0;
        while offset < data.len() {
            let brightness = match header.color_type {
                0 => {
                    let gray = data[offset] as f32;
                    gray
                }
                2 => {
                    let r = data[offset] as f32;
                    let g = data[offset + 1] as f32;
                    let b = data[offset + 2] as f32;
                    let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                    gray
                }
                3 => {
                    let palette_index = data[offset];
                    let gray = palette_index as f32;
                    gray
                }
                4 => {
                    let gray = data[offset] as f32;
                    gray
                }
                6 => {
                    let r = data[offset] as f32;
                    let g = data[offset + 1] as f32;
                    let b = data[offset + 2] as f32;
                    let gray = 0.299 * r + 0.587 * g + 0.114 * b;
                    gray
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid color type",
                    ));
                }
            };
            _brightness_data.push(brightness);
            offset += bytes_per_pixel;
        }
        let brightness_data = Brightness {
            shape: (header.width, header.height),
            data: _brightness_data,
        };
        Ok(Some(VisualData::Brightness(brightness_data)))
    }
}
