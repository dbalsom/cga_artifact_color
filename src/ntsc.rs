// Color sampling math adapted from https://www.shadertoy.com/view/Mdffz7 by xot

#![allow(unused)]

use std::time::Instant;

use cgmath::{Matrix3, Vector2, Vector3};

use crate::SampleMethod;

#[rustfmt::skip]
static YIQ2RGB: Matrix3<f32> = Matrix3::new(
    1.000, 1.000, 1.000, 
    0.956, -0.272, -1.106, 
    0.621, -0.647, 1.703,
);

const CCYCLE: i32 = 8;
const CCYCLE_HALF: i32 = CCYCLE / 2;

const N: usize = 15; //  Filter Width
const N2: usize = N * 2;
const M: usize = N / 2; //  Filter Middle
const M2: usize = N2 / 2;

const FC: f32 = 0.25; //  Frequency Cutoff
const SCF: f32 = 0.25; //  Subcarrier Frequency

const FC2: f32 = 0.125; //  Frequency Cutoff
const SCF2: f32 = 0.125; //  Subcarrier Frequency

const PI: f32 = 3.1415926;
const TAU: f32 = 6.2831853;

const SAT: f32 = 1.5;
const HUE: f32 = 2.0; // 0.5 Looks good for KQ, Speedway
const BRI: f32 = 1.0;

pub enum OutputType {
    Rgb,
    Luma,
    Chroma,
}

pub struct NTSCWeights {
    weights: [f32; N],
}
impl NTSCWeights {
    pub fn new() -> Self {
        let mut s = Self { weights: [0.0; N] };

        let mut sum: f32 = 0.0;
        for n in 0..N {
            s.weights[n] = hann(n as f32, N as f32) * sinc(FC * (n as i32 - M as i32) as f32);
            sum += s.weights[n];
        }
        // Normalize sampling weights
        for n in 0..N {
            s.weights[n] /= sum;
        }
        s
    }
}

pub struct NTSCWeightsWide {
    weights: [f32; N2],
}
impl NTSCWeightsWide {
    pub fn new() -> Self {
        let mut s = Self { weights: [0.0; N2] };

        let mut sum: f32 = 0.0;
        for n in 0..N2 {
            s.weights[n] = hann(n as f32, N2 as f32) * sinc(FC2 * (n as i32 - M2 as i32) as f32);
            sum += s.weights[n];
        }
        // Normalize sampling weights
        for n in 0..N2 {
            s.weights[n] /= sum;
        }
        s
    }
}

// Adjusts a YIQ color by hue, saturation and brightness factors
pub fn adjust(yiq: Vector3<f32>, h: f32, s: f32, b: f32) -> Vector3<f32> {
    #[rustfmt::skip]
    let m: Matrix3<f32> = Matrix3::new(
        b,0.0,0.0,
        0.0,s * h.cos(),-h.sin(),
        0.0,h.sin(),s * h.cos(),
    );

    m * yiq
}

pub fn hann(n: f32, nh: f32) -> f32 {
    return 0.5 * (1.0 - ((TAU * n) / (nh - 1.0)).cos());
}

pub fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        (x * PI).sin() / (x * PI)
    }
}

// Texture clamped sampling
pub fn sample_rgb_norm(img_in: &[u8], img_w: u32, img_h: u32, pos: Vector2<f32>) -> Vector3<f32> {
    let mut x = (pos.x * img_w as f32) as i32;
    let mut y = (pos.y * img_h as f32) as i32;

    if x < 0 {
        x = 0;
    }
    if y < 0 {
        y = 0;
    }
    if x >= img_w as i32 {
        x = img_w as i32 - 1;
    }
    if y >= img_h as i32 {
        y = img_h as i32 - 1;
    }

    let io = (y * img_w as i32 * 4 + x * 4) as usize;
    Vector3 {
        x: (img_in[io + 0] as f32 / 255.0),
        y: (img_in[io + 1] as f32 / 255.0),
        z: (img_in[io + 2] as f32 / 255.0),
    }
}

#[inline]
/// Return the RGB pixel at x, y, clamped at image dimensions
pub fn sample_rgb_xy(
    img_in: &[u8],
    img_w: u32,
    img_h: u32,
    mut x: i32,
    mut y: i32,
) -> Vector3<f32> {
    if x < 0 {
        x = 0;
    }
    if y < 0 {
        y = 0;
    }
    if x >= img_w as i32 {
        x = img_w as i32 - 1;
    }
    if y >= img_h as i32 {
        y = img_h as i32 - 1;
    }

    let io = (y * img_w as i32 * 4 + x * 4) as usize;
    Vector3 {
        x: (img_in[io + 0] as f32 / 255.0),
        y: (img_in[io + 1] as f32 / 255.0),
        z: (img_in[io + 2] as f32 / 255.0),
    }
}

#[inline]
/// Return the grayscale pixel at x, y, clamped at image dimensions
pub fn sample_gy_xy(img_in: &[u8], img_w: u32, img_h: u32, mut x: i32, mut y: i32) -> f32 {
    if x < 0 {
        x = 0;
    }
    if y < 0 {
        y = 0;
    }
    if x >= img_w as i32 {
        x = img_w as i32 - 1;
    }
    if y >= img_h as i32 {
        y = img_h as i32 - 1;
    }

    let io = (y * img_w as i32 + x) as usize;

    img_in[io] as f32 / 255.0
}

#[inline]
pub fn to_u8_clamped(f: f32) -> u8 {
    if f >= 255.0 {
        255
    } else if f <= 0.0 {
        0
    } else {
        f as u8
    }
}

pub fn process(
    img_in: &[u8],
    img_out: &mut [u8],
    img_w: u32,
    img_h: u32,
    hue: f32,
    sat: f32,
    luma: f32,
    method: SampleMethod,
    otype: OutputType,
) {
    //let weights = NTSCWeights::new();

    let pre_weight_t = Instant::now();
    let weights_w = NTSCWeightsWide::new();
    let weight_time = (Instant::now() - pre_weight_t).as_millis();

    log::debug!("Weight calculation took: {} ms", weight_time);

    let pre_process_t = Instant::now();

    match method {
        SampleMethod::Fast => {
            artifact_colors_fast(img_in, img_out, img_w, img_h, hue, sat, luma, otype);
        }
        SampleMethod::Accurate => {
            artifact_colors(
                img_in, img_out, img_w, img_h, hue, sat, luma, &weights_w, otype,
            );
        }
    }

    let process_time = (Instant::now() - pre_process_t).as_millis();

    log::debug!("Processing time took: {} ms", process_time);
}

pub fn sample_luma(img_in: &mut [u8], img_out: &mut [u8], img_w: u32, img_h: u32) {

    for y in 0..img_h {
        for x in 0..img_w {
            let mut yiq: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

            for n in -2..2 {

                let mut rgb = sample_rgb_xy(img_in, img_w, img_h, x as i32 + n, y as i32);
                yiq += rgb;
            }
            yiq = yiq / 4.0;

            let so = (y * img_w * 4 + x * 4) as usize;

            img_out[so + 0] = to_u8_clamped(yiq.x * 255.0);
            img_out[so + 1] = to_u8_clamped(yiq.y * 255.0);
            img_out[so + 2] = to_u8_clamped(yiq.z * 255.0);
            img_out[so + 3] = 255;
        }
    }
}

pub fn artifact_colors_fast(
    img_in: &[u8],
    img_out: &mut [u8],
    img_w: u32,
    img_h: u32,
    hue: f32,
    sat: f32,
    luma: f32,
    output_type: OutputType,
) {
    let mut sync_table: [(f32, f32, f32); 1280 + CCYCLE as usize] =
        [(0.0, 0.0, 0.0); 1280 + CCYCLE as usize];

    let pre_sync_t = Instant::now();
    // Precalculate sync
    for x in 0..(1280 + CCYCLE) {
        let phase: f32 = ((x - CCYCLE_HALF) as f32) * TAU / 8.0;
        sync_table[x as usize] = (phase, phase.cos(), phase.sin());
    }
    let sync_time = (Instant::now() - pre_sync_t).as_millis();
    log::debug!("Sync table took: {} ms", sync_time);

    let mut dst_o = 0;

    for y in 0..img_h {
        for x in 0..(img_w / 2) {
            let mut yiq: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

            for n in -CCYCLE_HALF..CCYCLE_HALF {
                let signal = sample_gy_xy(img_in, img_w, img_h, (x * 2) as i32 + n, y as i32);

                let sti = ((x * 2) as i32 + n as i32 + CCYCLE_HALF) as usize;
                let signal_i = signal * sync_table[sti].1;
                let signal_q = signal * sync_table[sti].2;

                //log::trace!("Sync: Calc: {},{} Table: {},{}", sync.y, sync.z, sync_table[sti].1, sync_table[sti].2);
                yiq.x += signal;
                yiq.y += signal_i;
                yiq.z += signal_q;
            }
            yiq = yiq / CCYCLE as f32;

            let adjust_yiq = adjust(yiq, hue, sat, luma);
            let rgb = YIQ2RGB * adjust_yiq;

            //let dst_o = (y * (img_w / 2) * 4 + (x / 2) * 4) as usize;
            match output_type {
                OutputType::Rgb => {
                    img_out[dst_o + 0] = to_u8_clamped(rgb.x * 255.0);
                    img_out[dst_o + 1] = to_u8_clamped(rgb.y * 255.0);
                    img_out[dst_o + 2] = to_u8_clamped(rgb.z * 255.0);
                }
                OutputType::Luma => {
                    img_out[dst_o + 0] = to_u8_clamped(yiq.x * 255.0);
                    img_out[dst_o + 1] = to_u8_clamped(yiq.x * 255.0);
                    img_out[dst_o + 2] = to_u8_clamped(yiq.x * 255.0);
                }
                OutputType::Chroma => {
                    img_out[dst_o + 0] = to_u8_clamped((40.0 * yiq.y + 0.5) * 255.0);
                    img_out[dst_o + 1] = to_u8_clamped((40.0 * yiq.z + 0.5) * 255.0);
                    img_out[dst_o + 2] = 0;
                }
            }
            img_out[dst_o + 3] = 255;

            dst_o += 4;
        }
    }
}

pub fn artifact_colors(
    img_in: &[u8],
    img_out: &mut [u8],
    img_w: u32,
    img_h: u32,
    hue: f32,
    sat: f32,
    luma: f32,
    weights: &NTSCWeightsWide,
    output_type: OutputType,
) {
    let iq: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

    for y in 0..img_h {
        for x in 0..(img_w / 2) {
            // Convert x, y coords to normalized form
            let uv: Vector2<f32> = Vector2 {
                x: x as f32 / img_w as f32,
                y: y as f32 / img_h as f32,
            };

            let mut yiq: Vector3<f32> = Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            for n in 0..N2 {
                // Position is calculated as (x + sample step - half filter width), this splits the sampling window across
                // the sampled pixel position
                let pos: Vector2<f32> = uv
                    + Vector2 {
                        x: uv.x + (n as i32 - M2 as i32) as f32 / (img_w as f32),
                        y: 0.0,
                    };

                let phase: f32 = TAU * (SCF2 * (img_w as f32) * pos.x);

                let mut signal = sample_gy_xy(
                    img_in,
                    img_w,
                    img_h,
                    (pos.x * (img_w - 1) as f32) as i32,
                    (pos.y * (img_h - 1) as f32) as i32,
                );

                let sync = Vector3::new(1.0, phase.cos(), phase.sin());
                yiq += Vector3::new(1.0, phase.cos(), phase.sin()) * signal * weights.weights[n];
            }

            let adjust_yiq = adjust(yiq, hue, sat, luma);
            let rgb = YIQ2RGB * adjust_yiq;

            let dst_o = (y * (img_w / 2) * 4 + (x) * 4) as usize;
            img_out[dst_o + 3] = 255;

            match output_type {
                OutputType::Rgb => {
                    img_out[dst_o + 0] = to_u8_clamped(rgb.x * 255.0);
                    img_out[dst_o + 1] = to_u8_clamped(rgb.y * 255.0);
                    img_out[dst_o + 2] = to_u8_clamped(rgb.z * 255.0);
                }
                OutputType::Luma => {
                    img_out[dst_o + 0] = to_u8_clamped(yiq.x * 255.0);
                    img_out[dst_o + 1] = to_u8_clamped(yiq.x * 255.0);
                    img_out[dst_o + 2] = to_u8_clamped(yiq.x * 255.0);
                }
                OutputType::Chroma => {
                    img_out[dst_o + 0] = to_u8_clamped((40.0 * yiq.y + 0.5) * 255.0);
                    img_out[dst_o + 1] = to_u8_clamped((40.0 * yiq.z + 0.5) * 255.0);
                    img_out[dst_o + 2] = 0;
                }
            }
        }
    }
}
