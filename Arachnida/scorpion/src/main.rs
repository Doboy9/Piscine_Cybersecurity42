use image::GenericImageView;
use rexif::parse_file;
use std::env;
use std::fs;

fn is_image_file(file_path: &str) -> bool {
    match image::open(file_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <image1> <image2> ...", args[0]);
        return;
    }

    for image_path in args.iter().skip(1) {
        if !fs::metadata(image_path).is_ok() {
            eprintln!("File does not exist: {}", image_path);
            continue;
        }

        if !is_image_file(image_path) {
            eprintln!("File is not a valid image: {}", image_path);
            continue;
        }

        match parse_file(image_path) {
            Ok(exif_data) => {
                println!("\nEXIF metadata for file: {}", image_path);

                for entry in exif_data.entries {
                    println!("{}: {}", entry.tag, entry.value_more_readable);
                }
            }
            Err(e) => eprintln!("Failed to read EXIF metadata: {}", e),
        }
    }
}
