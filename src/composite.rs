
#![allow (unused)]

use std::time::Instant;

pub const EDGE_RESPONSE: f32 = 0.80;
pub const INTENSITY_GAIN: f32 = 0.25;
pub const INTENSITY_GAIN_INT: u8 = 64;
pub const LUMA_ATTENUATE: f32 = 0.75;

#[derive(Copy, Clone, PartialEq)]
pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8
}

#[derive(Copy, Clone)]
pub struct YIQColor {
    y: f32,
    i: f32,
    q: f32
}

pub const CGA_RGB_TABLE: [RGBColor; 16] = [

    RGBColor { r:    0, g:    0, b:    0 },
    RGBColor { r:    0, g:    0, b: 0xAA },
    RGBColor { r:    0, g: 0xAA, b:    0 },
    RGBColor { r:    0, g: 0xAA, b: 0xAA },
    RGBColor { r: 0xAA, g:    0, b:    0 },
    RGBColor { r: 0xAA, g:    0, b: 0xAA },
    RGBColor { r: 0xAA, g: 0x55, b:    0 },
    RGBColor { r: 0xAA, g: 0xAA, b: 0xAA },

    RGBColor { r: 0x55, g: 0x55, b: 0x55 },
    RGBColor { r: 0x55, g: 0x55, b: 0xFF },
    RGBColor { r: 0x55, g: 0xFF, b: 0x55 },
    RGBColor { r: 0x55, g: 0xFF, b: 0xFF },
    RGBColor { r: 0xFF, g: 0x55, b: 0x55 },
    RGBColor { r: 0xFF, g: 0x55, b: 0xFF },
    RGBColor { r: 0xFF, g: 0xFF, b: 0x55 },
    RGBColor { r: 0xFF, g: 0xFF, b: 0xFF }
];

// Luma contribution of each color for each 1/2 Hdot of a color cycle
pub const COLOR_GEN_HALF: [[f32; 8]; 8] = [
    [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0 ], // Black
    [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0 ], // Blue
    [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0 ], // Green
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0 ], // Cyan
    [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0 ], // Red
    [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0 ], // Magenta
    [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0 ], // Yellow
    [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0 ], // White    
];

// Luma contribution of each color for each 1/2 Hdot of a color cycle
pub const COLOR_GEN_HALF_INT: [[u8; 8]; 8] = [
    [  0,   0,   0,   0,   0,   0,   0,   0 ], // Black
    [  0,   0,   0, 255, 255, 255, 255,   0 ], // Blue
    [255, 255,   0,   0,   0,   0, 255, 255 ], // Green
    [255,   0,   0,   0,   0, 255, 255, 255 ], // Cyan
    [  0, 255, 255, 255, 255,   0,   0,   0 ], // Red
    [  0,   0, 255, 255, 255, 255,   0,   0 ], // Magenta
    [255, 255, 255,   0,   0,   0,   0, 255 ], // Yellow
    [255, 255, 255, 255, 255, 255, 255, 255 ], // White    
];

pub const COLOR_GEN_EDGES_HALF: [[bool; 8]; 8] = [
    [false, false, false, false, false, false, false, false ], // Black
    [false, false, false, true,  false, false, true,  false ], // Blue
    [false, true,  false, false, false, false, true,  false ], // Green
    [true , false, false, false, false, true,  false, false ], // Cyan
    [false, true,  false, false, true,  false, false, false ], // Red
    [false, false, true,  false, false, true,  false, false ], // Magenta
    [false, false, true,  false, false, false, false, true  ], // Yellow
    [false, false, false, false, false, false, false, false ], // White    
];


/// Return the square of the distance between two colors in RGB space. Since we are just 
/// comparing the magnitude, we don't need to take the square root.
fn rgb_distance_squared(a: RGBColor, b: RGBColor) -> i32 {

    let dr = a.r as i32 - b.r as i32;
    let dg = a.g as i32 - b.g as i32;
    let db = a.b as i32 - b.b as i32;

    dr * dr + dg * dg + db * db
}

/// Convert an RGB value to a matching CGA color number.
/// We are just doing this in RGB space, which may not be the best method, but it seems to work okay.
/// There are only 16 possible target color values.
pub fn rgb_to_cga(rgb: RGBColor) -> u8 {

    let mut color_index = 0;
    let mut color_distance_rgb = 10000000;

    //let yiq_compare = rgb_to_yiq(rgb.r, rgb.g, rgb.b);

    if rgb == (RGBColor { r: 0, g: 0, b: 0 }) {
        return 0;
    }

    if rgb == (RGBColor { r:0xFF, g: 0xFF, b: 0xFF }) {
        return 15;
    }    

    for i in 0..16 {

        let rgb_palette = CGA_RGB_TABLE[i];

        let temp_distance = rgb_distance_squared(rgb, rgb_palette);

        if temp_distance < color_distance_rgb {
            color_index = i;
            color_distance_rgb = temp_distance;
        }
    }

    color_index as u8
}

/// Return the hdot number (0-3) for the given x position.
#[inline]
pub fn get_cycle_hdot(x: i32) -> usize {
    (x % 4).abs() as usize
}

/// Convert a slice of RGBA image data into a slice of CGA palette indices.
pub fn convert_rgb_to_cga_idx(img_in: &mut[u8], cga_out: &mut[u8], img_w: u32, img_h: u32) {

    for y in 0..img_h {
        for x in 0..img_w {

            let so: usize = ((y * img_w * 4) + (x * 4)) as usize;
            let co: usize = ((y * img_w) + x) as usize;

            let cga_idx = rgb_to_cga(
                RGBColor {
                    r: img_in[so + 0],
                    g: img_in[so + 1],
                    b: img_in[so + 2],
                }
            );
            cga_out[co] = cga_idx;
        }
    }
}

/// Convert a 640 pixel wide, 16 color CGA image into a 1280 pixel wide Composite image.
/// The input image should be a slice of RGBA pixel values.
/// The output image should be a slice of u8 values to receive the grayscale composite signal.
pub fn process_cga_composite(img_in: &mut [u8], img_out: &mut [u8], img_w: u32, img_h: u32) {

    let mut cga_buf: Vec<u8> = vec![0; (img_w * img_h) as usize];

    //let mut sample_slice: [u8; WINDOW_SIZE as usize] = [0; WINDOW_SIZE as usize];

    let bench_t = Instant::now();

    convert_rgb_to_cga_idx(img_in, &mut cga_buf, img_w, img_h);
    
    let ms = (Instant::now() - bench_t).as_millis();
    log::debug!("RGBA->CGA conversion time took: {}", ms);

    for y in 0..img_h {
        for x in 0..img_w {
            //get_sample_slice_cga(&cga_buf, img_w, img_h, x, y, &mut sample_slice);
            //let luma = get_cga_luma_avg_from_slice(&sample_slice, x as i32 - (WINDOW_SIZE / 2));

            let mut last_hhdot_value = 0.0;

            let src_o = (y * img_w + x) as usize;
            
            // Convert 0-15 color range to 0-7
            let color = cga_buf[src_o];
            let next_color = if x < (img_w - 1) {
                cga_buf[src_o + 1 as usize] % 8
            }
            else {
                0
            };
            let base_color = color % 8;
            let is_bright = color > 7;

            let hdot = get_cycle_hdot(x as i32);

            for h in 0..2usize {

                let mut attenuate = false;
                
                let mut hhdot_value = COLOR_GEN_HALF[base_color as usize][(hdot * 2 + h) as usize];
                let next_hhdot_value = match h {
                    0 => {
                        COLOR_GEN_HALF[base_color as usize][((hdot * 2 + h) + 1) % 8 as usize ]
                    }
                    _ => {
                        COLOR_GEN_HALF[next_color as usize][((hdot * 2 + h) + 1) % 8 as usize ]   
                    }
                };
                let hhdot_is_edge = COLOR_GEN_EDGES_HALF[base_color as usize][(hdot * 2 + h) as usize];

                if hhdot_value == 1.0 && last_hhdot_value == 0.0 {
                    // Signal is rising.
                    if hhdot_is_edge == true {
                        // Signal is rising with rising edge of color clock. Attenuate edge slew.
                        attenuate = true;
                    }
                }
                else if hhdot_value == 1.0 && next_hhdot_value == 0.0 {
                    // Signal is falling on next hhdot.
                    if hhdot_is_edge == true {
                        // Signal is falling with falling edge of color clock. Attenuate edge slew.
                        attenuate = true;
                    }
                }

                last_hhdot_value = hhdot_value;

                if attenuate {
                    hhdot_value *= EDGE_RESPONSE;
                }

                hhdot_value *= LUMA_ATTENUATE;

                if is_bright {
                    hhdot_value += INTENSITY_GAIN;
                }

                let composite_u8 = (hhdot_value * 255.0) as u8;
                let dst_o = ((y * img_w * 2) + (x * 2)) as usize;
                img_out[dst_o + h] = composite_u8;
            }
        }
    }
}


/// Convert a 640 pixel wide, 16 color CGA image into a 1280 pixel wide Composite image.
/// The input image should be a slice of RGBA pixel values.
/// The output image should be a slice of u8 values to receive the grayscale composite signal.
/// 
/// Uses integer math.
pub fn process_cga_composite_int(img_in: &mut [u8], img_out: &mut [u8], img_w: u32, img_h: u32) {

    let mut cga_buf: Vec<u8> = vec![0; (img_w * img_h) as usize];

    //let mut sample_slice: [u8; WINDOW_SIZE as usize] = [0; WINDOW_SIZE as usize];

    let mut bench_t = Instant::now();
    convert_rgb_to_cga_idx(img_in, &mut cga_buf, img_w, img_h);
    let us = (Instant::now() - bench_t).as_micros();
    log::debug!("RGBA->CGA conversion time took: {} milliseconds", us as f32 / 1000.0 );

    bench_t = Instant::now();

    let mut dst_o = 0;

    for y in 0..img_h {
        for x in 0..img_w {
            //get_sample_slice_cga(&cga_buf, img_w, img_h, x, y, &mut sample_slice);
            //let luma = get_cga_luma_avg_from_slice(&sample_slice, x as i32 - (WINDOW_SIZE / 2));

            let mut last_hhdot_value = 0;

            let src_o = (y * img_w + x) as usize;
            
            // Convert 0-15 color range to 0-7
            let color = cga_buf[src_o];
            let next_color = if x < (img_w - 1) {
                cga_buf[src_o + 1 as usize] % 8
            }
            else {
                0
            };
            let base_color = color % 8;
            let is_bright = color > 7;

            let hdot = get_cycle_hdot(x as i32);

            for h in 0..2usize {

                let mut attenuate = false;
                
                let mut hhdot_value = COLOR_GEN_HALF_INT[base_color as usize][(hdot * 2 + h) as usize];
                let next_hhdot_value = match h {
                    0 => {
                        COLOR_GEN_HALF_INT[base_color as usize][((hdot * 2 + h) + 1) % 8 as usize ]
                    }
                    _ => {
                        COLOR_GEN_HALF_INT[next_color as usize][((hdot * 2 + h) + 1) % 8 as usize ]   
                    }
                };
                let hhdot_is_edge = COLOR_GEN_EDGES_HALF[base_color as usize][(hdot * 2 + h) as usize];

                if hhdot_value == 255 && last_hhdot_value == 0 {
                    // Signal is rising.
                    if hhdot_is_edge == true {
                        // Signal is rising with rising edge of color clock. Attenuate edge slew.
                        attenuate = true;
                    }
                }
                else if hhdot_value == 255 && next_hhdot_value == 0 {
                    // Signal is falling on next hhdot.
                    if hhdot_is_edge == true {
                        // Signal is falling with falling edge of color clock. Attenuate edge slew.
                        attenuate = true;
                    }
                }

                last_hhdot_value = hhdot_value;

                /*
                if attenuate {
                    hhdot_value = ((hhdot_value as u32 * 768) >> 10) as u8;
                }
                */

                // Integer version of * 0.75
                hhdot_value = ((hhdot_value as u32 * 768) >> 10) as u8;

                if is_bright {
                    hhdot_value += INTENSITY_GAIN_INT;
                }
                
                //let dst_o = ((y * img_w * 2) + (x * 2)) as usize;
                img_out[dst_o + h] =  hhdot_value as u8;
                
            }
            dst_o += 2;
        }
    }

    let us = (Instant::now() - bench_t).as_micros();
    log::debug!("Composite conversion took: {} milliseconds", us as f32 / 1000.0 );
}