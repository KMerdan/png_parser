// main.rs
use std::env;
use std::io;

mod png;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.png>", args[0]);
        return Ok(());
    }
    let file_path = &args[1];
    let mut reader = png::PngReader::new(file_path)?;
    let raw_png = reader.read_png()?;
    println!("Header: {:?}", raw_png.header);
    for chunk in raw_png.chunks {
        println!("{}", chunk);
    }
    Ok(())
}
