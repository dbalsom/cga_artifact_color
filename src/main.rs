/*
    cga_artifact_color
    https://github.com/dbalsom/cga_artifact_color/

    Copyright 2022-2023 Daniel Balsom

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER   
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.

    --------------------------------------------------------------------------
*/

#![allow(unused)]

use std::path::PathBuf;
use std::time::Instant;
use std::str::FromStr;

use bytemuck;
use bpaf::{Bpaf, Parser};
use image;
use image::imageops::FilterType;

mod composite;
mod ntsc;
mod reenigne_composite;

use ntsc::OutputType;
use reenigne_composite::{ReCompositeContext, ReCompositeBuffers};

#[derive (Copy, Clone, Debug, Bpaf)]
pub enum SampleMethod {
    Fast,
    Accurate,
    Reenigne
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
            "reenigne" => Ok(SampleMethod::Reenigne),
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

    let mut img_w = img.width();
    let mut img_h = img.height();

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

            img_w = rgba8_img.width();
            img_h = rgba8_img.height();

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

    // Do reenigne convert.
    if let SampleMethod::Reenigne = shell_args.method {
        
        let mut comp_ctx = ReCompositeContext::new();
        //comp_ctx.update_cga16_color(0b1_0110); // hires graphics
        comp_ctx.update_cga16_color(0b0_0001); // 80 col text mode graphics

        comp_ctx.print();

        // Create buffers.
        let mut comp_buf = ReCompositeBuffers::new();

        // Convert RGB source image to indexed color.
        let mut cga_buf: Vec<u8> = vec![0; (img_w * img_h) as usize];
        composite::convert_rgb_to_cga_idx(&mut bytes_in, &mut cga_buf, img_w, img_h);

        // Convert every row of source image.

        let rgba_out32: &mut [u32] = bytemuck::cast_slice_mut(&mut rgba_out);

        rgba_out32.fill(0xFFFFFFFF);

        // Bench reenigne composite
        let bench_t = Instant::now();

        for y in 0..img_h {

            let yo = (y * img_w) as usize;
            let in_slice = &mut cga_buf[yo..(yo + img_w as usize)];
            let out_slice = &mut rgba_out32[yo..(yo + img_w as usize)];

            comp_ctx.composite_process(0, img_w as usize, &mut comp_buf, in_slice, out_slice);
        }

        let us = (Instant::now() - bench_t).as_micros();
        let ms = us as f64 / 1000.0;
        log::debug!("reenigne composite took: {} ms", ms);

        match image::save_buffer(
            "./out_reenigne.png",
            &rgba_out,
            img_w,
            img_h,
            image::ColorType::Rgba8,
        ) {
            Ok(_) => println!("Wrote out_reenigne.png!"),
            Err(e) => {
                println!("Error writing output file: {}", e)
            }
        }     

        return;
    }

    // Non-reenigne methods

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
