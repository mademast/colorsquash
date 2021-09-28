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
    for color in image.pixels_mut() {
        let quantized = color_map.entry(*color).or_insert({
            let mut min_difference = f32::MAX;
            let mut min_difference_color = *color;

            for selected_color in &selected_colors {
                let difference = rgb_difference(color, selected_color);
                if difference < min_difference {
                    min_difference = difference;
                    min_difference_color = *selected_color;
                }
            }
            min_difference_color
        });

        *color = *quantized;
    }

    image.save(outname).expect("Failed to write out");
}

fn quantize<'a, T>(pixels: T) -> Vec<Rgb<u8>>
where
    T: Iterator<Item = &'a Rgb<u8>>,
{
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

    let mut selected_colors: Vec<Rgb<u8>> = Vec::with_capacity(MAX_COLORS);

    for (key, _value) in sorted.iter() {
        if selected_colors.len() >= MAX_COLORS {
            break;
        } else if selected_colors
            .iter()
            .all(|color| rgb_difference(key, color) > RGB_TOLERANCE)
        {
            selected_colors.push(*key);
        }
    }

    selected_colors
}

#[allow(clippy::many_single_char_names)]
fn rgb_difference(a: &Rgb<u8>, z: &Rgb<u8>) -> f32 {
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
