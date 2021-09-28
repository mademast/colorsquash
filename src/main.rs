use std::{collections::HashMap, env::args};

use image::io::Reader as ImageReader;
use image::{buffer::Pixels, Rgb};

const MAX_COLORS: usize = 256;

const TOLERANCE: f32 = 0.6;
const RGB_TOLERANCE: f32 = 10.0 * TOLERANCE;

fn main() {
    let filename = args().nth(1).unwrap();
    let outname = args().nth(2).unwrap();
    // The percent of RGB value difference a color has to surpass to be considered unique

    let imageread = ImageReader::open(&filename).expect("Failed to open image!");
    let mut image = imageread
        .decode()
        .expect("Failed to decode image!")
        .into_rgb8();

    let selected_colors = quantize(image.pixels());

    let mut color_map: HashMap<Rgb<u8>, Rgb<u8>> = HashMap::with_capacity(image.len() / 2);
    // Selected colors are themselves
    for color in selected_colors.iter() {
        color_map.insert(*color, *color);
    }

    // Max complexity is O(n * max_colors)
    for color in image.pixels() {
        let mut min_difference = f32::MAX;
        let mut min_difference_color = *color;

        for index in 0..selected_colors.len() {
            let difference = rgb_difference(color, unsafe { selected_colors.get_unchecked(index) });
            /*if difference == 0.0 {
                continue 'sorted_colors;
            }*/
            if difference < min_difference {
                min_difference = difference;
                min_difference_color = unsafe { *selected_colors.get_unchecked(index) };
            }
        }

        color_map.insert(*color, min_difference_color);
    }

    for pixel in image.pixels_mut() {
        pixel.clone_from(color_map.get(pixel).unwrap());
    }

    image.save(outname).expect("Failed to write out");
}

fn quantize(pixels: Pixels<Rgb<u8>>) -> Vec<Rgb<u8>> {
    let mut colors: HashMap<Rgb<u8>, usize> = HashMap::new();

    //count pixels
    for pixel in pixels {
        match colors.get_mut(pixel) {
            None => {
                colors.insert(*pixel, 1);
            }
            Some(n) => *n += 1,
        }
    }

    let mut sorted: Vec<(Rgb<u8>, usize)> = colors.into_iter().collect();
    sorted.sort_by(|(colour1, freq1), (colour2, freq2)| {
        freq2
            .cmp(freq1)
            .then(colour2[0].cmp(&colour1[0]))
            .then(colour2[1].cmp(&colour1[1]))
            .then(colour2[2].cmp(&colour1[2]))
    });

    println!("Sorted! Selecting colors...");

    let mut sorted_iter = sorted.iter();

    let mut selected_colors: Vec<Rgb<u8>> = Vec::with_capacity(MAX_COLORS);
    selected_colors.push(sorted_iter.next().unwrap().0);

    for (key, _value) in sorted_iter {
        if selected_colors.len() < MAX_COLORS {
            for selected_color in selected_colors.iter() {
                if rgb_difference(key, selected_color) > RGB_TOLERANCE {
                    selected_colors.push(*key);
                    break;
                }
            }
        } else {
            break;
        }
    }

    selected_colors
}
#[allow(clippy::many_single_char_names)]
fn rgb_difference(a: &Rgb<u8>, z: &Rgb<u8>) -> f32 {
    //((a.0[0] as i16 - b.0[0] as i16).abs() + (a.0[1] as i16 - b.0[1] as i16).abs() +(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
    //(a.0[0] as i16 - b.0[0] as i16).abs().max((a.0[1] as i16 - b.0[1] as i16).abs().max(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
    //(a.0[0] as i16 - b.0[0] as i16).abs().max((a.0[1] as i16 - b.0[1] as i16).abs()).max((a.0[2] as i16 - b.0[2] as i16).abs()) as u16
    /*(((a.0[0] as i32 - b.0[0] as i32) * (a.0[0] as i32 - b.0[0] as i32))
    + ((a.0[1] as i32 - b.0[1] as i32) * (a.0[1] as i32 - b.0[1] as i32))
    + ((a.0[2] as i32 - b.0[2] as i32) * (a.0[2] as i32 - b.0[2] as i32)))
    .abs() as u16*/
    let (a, b, c) = pixel_rgb_to_hsv(a);
    let (d, e, f) = pixel_rgb_to_hsv(z);

    (((c - f) * (c - f)) + ((a - d).abs() / 90.0) + (b - e).abs()) as f32
}

#[allow(clippy::float_cmp)]
fn pixel_rgb_to_hsv(a: &Rgb<u8>) -> (f32, f32, f32) {
    let (r, g, b) = (
        a.0[0] as f32 / 256.0,
        a.0[1] as f32 / 256.0,
        a.0[2] as f32 / 256.0,
    );

    let value = r.max(g.max(b));
    let x_min = r.min(g.min(b));
    let chroma = value - x_min;

    let hue = if chroma == 0.0 {
        0.0
    } else if value == r {
        60.0 * ((g - b) / chroma)
    } else if value == g {
        60.0 * (2.0 + (b - r) / chroma)
    } else if value == b {
        60.0 * (4.0 + (r - g) / chroma)
    } else {
        unreachable!()
    };

    let value_saturation = if value == 0.0 { 0.0 } else { chroma / value };

    /* Rotate the color wheel counter clockwise to the negative location
          |       Keep the wheel in place and remove any full rotations
     _____V____ _____V____
    |          |          |*/
    ((hue + 360.0) % 360.0, value_saturation * 2.0, value * 2.0)
}
