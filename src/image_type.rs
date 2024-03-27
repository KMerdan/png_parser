use core::panic;
use std::fmt::Display;

const DENSITY_CHAR: [char; 9] = ['.', ',', ':', '+', '*', '?', '%', '#', '@'];

pub struct CharImage {
    pub shape: (u32, u32),
    pub data: Vec<char>,
}

pub struct PNG {
    pub shape: (u32, u32),
    pub data: Vec<u8>,
}
impl Display for PNG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PNG image with shape: {:?} and {} bytes of data",
            self.shape,
            self.data.len()
        )
    }
}

pub struct Brightness {
    pub shape: (u32, u32),
    pub data: Vec<f32>,
}
impl Display for Brightness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Brightness data with shape: {:?} and {} bytes of data",
            self.shape,
            self.data.len()
        )
    }
}

pub enum VisualData {
    RGBA(PNG),
    Brightness(Brightness),
    Charimage(CharImage),
}
impl Display for VisualData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisualData::RGBA(png) => write!(f, "{}", png),
            VisualData::Brightness(brightness) => write!(f, "{}", brightness),
            VisualData::Charimage(char_image) => {
                let ascii_image: Vec<char> = self.into();
                let ascii_image: Vec<_> = ascii_image.chunks(char_image.shape.1 as usize).collect();
                let mut result = String::new();
                for row in ascii_image {
                    for &ch in row {
                        result.push(ch);
                    }
                    result.push('\n');
                }
                write!(f, "{}", result)
            }
        }
    }
}

impl Into<Vec<u8>> for &VisualData {
    fn into(self) -> Vec<u8> {
        match self {
            VisualData::RGBA(png) => png.data.clone(),
            VisualData::Brightness(_) => panic!("Cannot convert Brightness to PNG"),
            VisualData::Charimage(_) => panic!("Cannot convert CharImage to PNG"),
        }
    }
}

impl Into<Vec<f32>> for &VisualData {
    fn into(self) -> Vec<f32> {
        match self {
            VisualData::RGBA(_) => panic!("Cannot convert PNG to Brightness"),
            VisualData::Brightness(brightness) => brightness.data.clone(),
            VisualData::Charimage(_) => panic!("Cannot convert CharImage to Brightness"),
        }
    }
}

impl Into<Vec<char>> for &VisualData {
    fn into(self) -> Vec<char> {
        match self {
            VisualData::RGBA(_) => panic!("Cannot convert PNG to CharImage"),
            VisualData::Charimage(char_image) => char_image.data.clone(),
            VisualData::Brightness(brightness) => {
                let mut char_image = CharImage {
                    shape: brightness.shape,
                    data: Vec::new(),
                };
                let max: f32 = brightness.data.iter().fold(0.0, |acc, &x| acc.max(x));
                let min: f32 = brightness.data.iter().fold(1.0, |acc, &x| acc.min(x));
                let range = max - min;
                let step = range / 9.0;
                for i in 0..brightness.data.len() {
                    let index = ((brightness.data[i] - min) / step).floor() as usize;
                    let index = if index >= DENSITY_CHAR.len() {
                        DENSITY_CHAR.len() - 1
                    } else {
                        index
                    };
                    char_image.data.push(DENSITY_CHAR[index]);
                }
                println!("{:?}", char_image.shape);
                char_image.data
            }
        }
    }
}
