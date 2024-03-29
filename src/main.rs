// main.rs
use std::env;
use std::io;

mod chunk;
mod error;
mod header;
mod image_type;
mod png;
mod raw_data;

use error::PngError;

fn main() -> Result<(), PngError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.png> <print_size>", args[0]);
        return Ok(());
    }

    let file_path = &args[1];
    let print_size = args[2].parse::<usize>().unwrap_or(20);

    let mut reader = png::PngReader::new(file_path)?;
    let raw_png = reader.load_png()?;
    println!("{}", raw_png);
    for chunk in &raw_png.chunks {
        println!("{}", chunk);
    }
    if let Some(visual_data) = reader.to_brightness_data(&raw_png, print_size)? {
        let char_vec: Vec<char> = (&visual_data).into();
        let char_string: &String = &(char_vec.iter().collect());
        println!("{}", char_string);
    } else {
        eprintln!("Failed to convert to brightness data");
    }

    // println!("{}", visual_data.unwrap());

    Ok(())
}
