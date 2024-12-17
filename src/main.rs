use clap::Parser;
use crossterm::{
    queue,
    style::{Color, ResetColor, SetForegroundColor},
};
use image::{DynamicImage, GenericImageView, Pixel};
use std::io::{self, Write};
use std::process::exit;

const ASCII_CHARS: &str = "@#8&$%*+;:,. ";

#[derive(Parser)]
#[command(
    name = "jpg_to_ascii",
    version = "1.0",
    about = "JPG to ASCII Art Converter"
)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value_t = 64)] // デフォルトの幅を64文字に設定
    width: u32,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let ascii_art = image_to_ascii(&args.input, args.width)?;
    print_colored_ascii(&ascii_art, &args.input)?;
    Ok(())
}

/// 画像をアスキーアートに変換する関数
fn image_to_ascii(input_path: &str, new_width: u32) -> Result<String, Box<dyn std::error::Error>> {
    let img = load_image(input_path)?;
    let (orig_width, orig_height) = img.dimensions();

    // アスペクト比を維持して高さを計算
    let aspect_ratio = orig_height as f32 / orig_width as f32;
    let new_height = (new_width as f32 * aspect_ratio * 0.55) as u32;

    let gray_image = img
        .resize_exact(new_width, new_height, image::imageops::FilterType::Nearest)
        .to_luma8();

    let ascii_art = gray_image
        .rows()
        .map(|row| {
            row.map(|pixel| {
                let intensity = pixel[0] as f32 / 255.0;
                ASCII_CHARS
                    .chars()
                    .nth((intensity * (ASCII_CHARS.len() - 1) as f32).round() as usize)
                    .unwrap()
            })
            .collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(ascii_art)
}

/// アスキーアートに色をつけて表示する関数
fn print_colored_ascii(
    ascii_art: &str,
    input_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = load_image(input_path)?;
    let (orig_width, _) = img.dimensions();
    let mut handle = io::stdout().lock();

    ascii_art.chars().enumerate().for_each(|(char_idx, ch)| {
        let (x, y) = (
            char_idx % orig_width as usize,
            char_idx / orig_width as usize,
        );
        if let Some(pixel) = img.get_pixel(x as u32, y as u32).to_rgba().0.get(0..3) {
            let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
            queue!(handle, SetForegroundColor(Color::Rgb { r, g, b })).unwrap();
            write!(handle, "{}", ch).unwrap();
        }
    });

    queue!(handle, ResetColor).unwrap();
    handle.flush().unwrap();
    Ok(())
}

/// 画像を読み込む関数
fn load_image(input_path: &str) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    image::open(input_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
