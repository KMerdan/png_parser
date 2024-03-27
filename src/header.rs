// header.rs
use std::fmt;

#[derive(Debug)]
pub struct HeaderInfo {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

impl HeaderInfo {
    pub fn new(data: &[u8]) -> Result<Option<HeaderInfo>, String> {
        if data.len() < 13 {
            return Err("Invalid data: Insufficient bytes for header information".to_string());
        }

        let width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let bit_depth = data[8];
        let color_type = data[9];
        let compression_method = data[10];
        let filter_method = data[11];
        let interlace_method = data[12];

        Ok(Some(HeaderInfo {
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        }))
    }

    pub fn to_string(&self) -> String {
        let color_type = match self.color_type {
            0 => "Grayscale",
            2 => "R,G,B triple",
            3 => "Indexed-color",
            4 => "Grayscale with alpha",
            6 => "R,G,B triple with alpha",
            _ => "Unknown",
        };

        let compression_method = match self.compression_method {
            0 => "Deflate/inflate",
            _ => "Unknown",
        };

        let filter_method = match self.filter_method {
            0 => "None",
            1 => "Sub",
            2 => "Up",
            3 => "Average",
            4 => "Paeth",
            _ => "Unknown",
        };

        let interlace_method = match self.interlace_method {
            0 => "None",
            1 => "Adam7",
            _ => "Unknown",
        };

        format!(
            "\n\tWidth: {}\n\tHeight: {}\n\tBit depth: {}\n\tColor type: {}\n\tCompression method: {}\n\tFilter method: {}\n\tInterlace method: {}",
            self.width, self.height, self.bit_depth, color_type, compression_method, filter_method, interlace_method
        )
    }
}
