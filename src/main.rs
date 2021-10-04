use std::time::Instant;
use std::{collections::HashMap, env::args};

use image::io::Reader as ImageReader;
use image::Rgb;

use ahash::RandomState;

use rayon::prelude::*;

const MAX_COLORS: usize = 16;

const RGB_TOLERANCE: f32 = 0.25 + (1.0 - (MAX_COLORS as f32 / 256.0));

fn main() {
    let filename = args().nth(1).unwrap();
    let outname = args().nth(2).unwrap();
    // The percent of RGB value difference a color has to surpass to be considered unique

    let imageread = ImageReader::open(&filename).expect("Failed to open image!");
    let mut image = imageread
        .decode()
        .expect("Failed to decode image!")
        .into_rgb8();

    //let mem_before_sort = mallinfo().hblkhd as usize;
    let start_sort = Instant::now();
    let sorted_colors = unique_and_sort(image.pixels());
    println!("Sort took {}s", start_sort.elapsed().as_secs_f32());

    //let mem_before_selection = mallinfo().hblkhd as usize;
    let start_selection = Instant::now();
    let selected_colors = select_colors(&sorted_colors);
    println!(
        "Color Selection took {}s. Count {}",
        start_selection.elapsed().as_secs_f32(),
        selected_colors.len()
    );

    let start_array = Instant::now();
    let mut array = vec![0usize; 256 * 256 * 256];
    println!(
        "Array creation took {}s",
        start_array.elapsed().as_secs_f32()
    );

    let start_map = Instant::now();

    for (sorted, _) in &sorted_colors {
        let mut min_diff = f32::MAX;
        let mut min_index = usize::MAX;

        for (index, selected) in selected_colors.iter().enumerate() {
            let diff = rgb_difference(sorted, selected);
            if diff < min_diff {
                min_diff = diff;
                min_index = index;
            }
        }

        array[color_index(sorted)] = min_index;
    }

    println!(
        "Creating color map {:.2}s",
        start_map.elapsed().as_secs_f32()
    );

    let start_fill = Instant::now();
    // Max complexity is O(n * max_colors)
    for color in image.pixels_mut() {
        let index = array[color_index(color)];

        *color = selected_colors[index];
    }
    println!(
        "Took {:.2}s to fill in the image.\nTotal time from sort {:.2}s",
        start_fill.elapsed().as_secs_f32(),
        start_sort.elapsed().as_secs_f32()
    );

    image.save(outname).expect("Failed to write out");
}

fn unique_and_sort<'a, T>(pixels: T) -> Vec<(Rgb<u8>, usize)>
where
    T: Iterator<Item = &'a Rgb<u8>>,
{
    let mut colors: HashMap<Rgb<u8>, usize, RandomState> = HashMap::default();

    //count pixels
    for pixel in pixels {
        match colors.get_mut(pixel) {
            None => {
                colors.insert(*pixel, 1);
            }
            Some(n) => *n += 1,
        }
    }

    let mut sorted: Vec<(Rgb<u8>, usize)> = colors.into_par_iter().collect();
    sorted.sort_by(|(colour1, freq1), (colour2, freq2)| {
        freq2
            .cmp(freq1)
            .then(colour2[0].cmp(&colour1[0]))
            .then(colour2[1].cmp(&colour1[1]))
            .then(colour2[2].cmp(&colour1[2]))
    });

    sorted
}

fn select_colors(sorted: &[(Rgb<u8>, usize)]) -> Vec<Rgb<u8>> {
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

#[inline(always)]
fn color_index(c: &Rgb<u8>) -> usize {
    c.0[0] as usize * (256 * 256) + c.0[1] as usize * 256 + c.0[2] as usize
}

#[allow(clippy::many_single_char_names)]
#[inline(always)]
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
