use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn read_png_file(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.png>", args[0]);
        eprint!("No input file specified");
        return;
    }
    let file_path = &args[1];
    match read_png_file(file_path) {
        Ok(buffer) => {
            println!("Read {} bytes from {}", buffer.len(), file_path);
        }
        Err(error) => {
            eprintln!("Error reading {}: {}", file_path, error);
        }
    }
}

// fn main() {
//     let file_path = "./data/fun.png";
//     match read_png_file(file_path) {
//         Ok(buffer) => {
//             println!("Read {} bytes from {}", buffer.len(), file_path);
//         }
//         Err(error) => {
//             eprintln!("Error reading {}: {}", file_path, error);
//         }
//     }
// }
