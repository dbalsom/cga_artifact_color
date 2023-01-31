#![allow(unused)]

use std::path::PathBuf;
use std::time::Instant;

use image;
use image::imageops::FilterType;


use std::str::FromStr;

use bpaf::{Bpaf, Parser};

mod composite;
mod ntsc;

use ntsc::OutputType;

#[derive (Copy, Clone, Debug, Bpaf)]
pub enum SampleMethod {
    Fast,
    Accurate,
}

impl FromStr for SampleMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        match s.to_lowercase().as_str() {
            "fast" => Ok(SampleMethod::Fast),
            "accurate" => Ok(SampleMethod::Accurate),
            _ => Err("Bad value for validatortype".to_string()),
        }
    }
}


#[derive(Debug, Bpaf)]
#[bpaf(version, generate(cli_args))]
pub struct CmdLineArgs {
    #[bpaf(long)]
    pub input: PathBuf,

    #[bpaf(long, short)]
    pub hue: f32,

    #[bpaf(long, short)]
    pub sat: f32,

    #[bpaf(long, short)]
    pub luma: f32,

    #[bpaf(long)]
    pub method: SampleMethod
}

fn main() {
    env_logger::init();

    let shell_args: CmdLineArgs = cli_args().run();

    let img = image::open(shell_args.input.clone()).unwrap_or_else(|e| {
        eprintln!("Couldn't open input image '{:?}': {}", shell_args.input, e);
        std::process::exit(1);
    });

    let img_w = img.width();
    let img_h = img.height();

    //let rgba8_img = img.to_rgba8();

    //println!("Loaded ({} x {}) image of length {}.", img_w, img_h, bytes_in.len());

    let mut out_width_factor = 1;
    let rgba8_img;
    let mut bytes_in;
    let mut rgba_out;

    match img_w {
        320 => {
            // Resize to 640 before processing.
            rgba8_img = img
                .resize(img_w * 2, img_h * 2, FilterType::Nearest)
                .to_rgba8();

            bytes_in = rgba8_img.into_raw();
            rgba_out = vec![0; bytes_in.len()];
        }
        640 => {
            rgba8_img = img.to_rgba8();
            bytes_in = rgba8_img.into_raw();

            rgba_out = vec![0; bytes_in.len()];
        }
        _ => {
            println!("Unsupported image width: {}", img_w);
            std::process::exit(1);
        }
    }

    // First, convert the input image to a composite signal. The resulting image will be grayscale (/4) of twice the horizontal resolution (*2)
    let mut composite_out = vec![0; bytes_in.len() / 2];
    let pre_composite_t = Instant::now();

    //composite::process_cga_composite(&mut bytes_in, &mut composite_out, img_w, img_h);
    composite::process_cga_composite_int(&mut bytes_in, &mut composite_out, img_w, img_h);

    let composite_time = (Instant::now() - pre_composite_t).as_millis();
    log::debug!("Composite conversion took: {} ms.", composite_time);

    match image::save_buffer(
        "./out_composite.png",
        &composite_out,
        img_w * 2,
        img_h,
        image::ColorType::L8,
    ) {
        Ok(_) => println!("Wrote out_composite.png!"),
        Err(e) => {
            println!("Error writing output file: {}", e)
        }
    }

    ntsc::process(
        &composite_out,
        &mut rgba_out,
        img_w * 2,
        img_h,
        shell_args.hue,
        shell_args.sat,
        shell_args.luma,
        shell_args.method,
        OutputType::Rgb,
    );

    match image::save_buffer(
        "./out.png",
        &rgba_out,
        img_w,
        img_h,
        image::ColorType::Rgba8,
    ) {
        Ok(_) => println!("Wrote out.png!"),
        Err(e) => {
            println!("Error writing output file: {}", e)
        }
    }

    ntsc::process(
        &composite_out,
        &mut rgba_out,
        img_w * 2,
        img_h,
        shell_args.hue,
        shell_args.sat,
        shell_args.luma,        
        shell_args.method,
        OutputType::Luma,
    );

    match image::save_buffer(
        "./out_luma.png",
        &rgba_out,
        img_w,
        img_h,
        image::ColorType::Rgba8,
    ) {
        Ok(_) => println!("Wrote out_luma.png!"),
        Err(e) => {
            println!("Error writing output file: {}", e)
        }
    }

    ntsc::process(
        &composite_out,
        &mut rgba_out,
        img_w * 2,
        img_h,
        shell_args.hue,
        shell_args.sat,
        shell_args.luma,
        shell_args.method,
        OutputType::Chroma,
    );

    match image::save_buffer(
        "./out_chroma.png",
        &rgba_out,
        img_w,
        img_h,
        image::ColorType::Rgba8,
    ) {
        Ok(_) => println!("Wrote out_chroma.png!"),
        Err(e) => {
            println!("Error writing output file: {}", e)
        }
    }
}
