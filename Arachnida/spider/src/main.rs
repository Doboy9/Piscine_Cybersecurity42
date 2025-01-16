use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::copy;
use std::path::Path;

struct Options {
    r: bool,
    l: u8,
    p: String,
    url: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut my_options = Options {
        r: false,
        l: 0,
        p: "./data/".to_string(),
        url: "".to_string(),
    };

    if args.iter().filter(|&arg| arg == "-r").count() > 1
        || args.iter().filter(|&arg| arg == "-p").count() > 1
        || args.iter().filter(|&arg| arg == "-l").count() > 1
    {
        println!("Only one -r, -l or -p authorized");
        return;
    }

    if args.contains(&"-r".to_string()) {
        println!("rrr");
        my_options.r = true;
    }
    if args.contains(&"-r".to_string()) {
        my_options.r = true;
        if args.contains(&"-l".to_string()) {
            if let Some(index) = args.iter().position(|arg| arg == "-l") {
                if index + 1 < args.len() {
                    println!("Argument after -l: {}", args[index + 1]);
                    if let Ok(value) = args[index + 1].parse::<u8>() {
                        my_options.l = value;
                    }
                }
            }
        } else {
            my_options.l = 5;
            println!("Default recursive depth set to {}", my_options.l);
        }
    }

    if args.contains(&"-p".to_string()) {
        if let Some(index) = args.iter().position(|arg| arg == "-p") {
            if index + 1 < args.len() {
                my_options.p = args[index + 1].clone();
                let path = Path::new(&my_options.p);
                if let Err(e) = fs::create_dir_all(path) {
                    println!("Path not available {}", e);
                } else {
                    my_options.p = args[index + 1].clone();
                    println!("{}", &my_options.p);
                }
            }
        }
        println!("pppp");
    }
    my_options.url = args[args.len() - 1].clone();
    println!("{}", my_options.url);
    exec(my_options);
}

fn exec(options: Options) {
    let mut visited_urls = HashSet::new();
    if options.r {
        download_images_recursively(&options.url, &options.p, options.l, &mut visited_urls);
    } else {
        download_and_save_image(&options.url, &options.p);
    }
}

fn download_images_recursively(
    url: &str,
    save_dir: &str,
    depth: u8,
    visited: &mut HashSet<String>,
) {
    if depth == 0 || visited.contains(url) {
        return;
    }

    visited.insert(url.to_string());

    if is_valid_image(url) {
        println!("Downloading direct image: {}", url);
        download_and_save_image(url, save_dir);
        return;
    }

    match get(url) {
        Ok(response) => {
            let body = response.text().unwrap();
            let document = Html::parse_document(&body);
            let img_selector = Selector::parse("img").unwrap();
            let link_selector = Selector::parse("a").unwrap();

            for img in document.select(&img_selector) {
                if let Some(src) = img.value().attr("src") {
                    let img_url = if src.starts_with("http") {
                        src.to_string()
                    } else {
                        format!(
                            "{}/{}",
                            url.trim_end_matches('/'),
                            src.trim_start_matches('/')
                        )
                    };
                    if is_valid_image(&img_url) {
                        download_and_save_image(&img_url, save_dir);
                    }
                }
            }

            for link in document.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    let link_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!(
                            "{}/{}",
                            url.trim_end_matches('/'),
                            href.trim_start_matches('/')
                        )
                    };
                    download_images_recursively(&link_url, save_dir, depth - 1, visited);
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch the page: {}", e);
        }
    }
}

fn download_and_save_image(url: &str, save_dir: &str) {
    let filename = url.split('/').last().unwrap();
    let filepath = format!("{}/{}", save_dir, filename);

    if Path::new(&filepath).exists() {
        println!("File already exists, skipping: {}", filepath);
        return;
    }

    match get(url) {
        Ok(response) => {
            let mut file = match File::create(&filepath) {
                Ok(ok) => ok,
                Err(e) => {
                    println!("Failed to create the file: {}", e);
                    return;
                }
            };
            let content = response.bytes().unwrap();
            if let Err(e) = copy(&mut content.as_ref(), &mut file) {
                println!("Failed to copy content: {}", e);
            } else {
                println!("Image downloaded successfully: {}", filepath);
            }
        }
        Err(e) => {
            println!("Failed to download the image: {}", e);
        }
    }
}

fn is_valid_image(url: &str) -> bool {
    let valid_extensions = [".jpg", ".jpeg", ".png", ".gif", ".bmp"];
    valid_extensions.iter().any(|ext| url.ends_with(ext))
}
