use std::{collections::HashMap, env::args, time::Instant};

use image::io::Reader as ImageReader;
use image::Rgb;

fn main() {
    let before = Instant::now();
    let filename = args().nth(1).unwrap();
    let outname = args().nth(2).unwrap();
    // The percent of RGB value difference a color has to surpass to be considered unique
    let tolerance = 0.6;
    let rgb_tolerance = 10.0 * tolerance;
    let max_colors = 256;

    println!("File is {}", &filename);

    let imageread = ImageReader::open(&filename).expect("Failed to open image!");
    let mut image = imageread
        .decode()
        .expect("Failed to decode image!")
        .into_rgb8();

    println!("Decoded!");
    let before_algo = Instant::now();

    let mut colors: HashMap<Rgb<u8>, usize> = HashMap::new();

    //count pixels
    for pixel in image.pixels() {
        match colors.get_mut(pixel) {
            None => {
                colors.insert(*pixel, 1);
            }
            Some(n) => *n += 1,
        }
    }

    println!(
        "{} has {} colors in it. Sorting most occuring to least...",
        filename,
        colors.len()
    );

    let mut sorted: Vec<(Rgb<u8>, usize)> = colors.into_iter().collect();
    sorted.sort_by(|(colour1, freq1), (colour2, freq2)| {
        freq2
            .cmp(freq1)
            .then(colour2[0].cmp(&colour1[0]))
            .then(colour2[1].cmp(&colour1[1]))
            .then(colour2[2].cmp(&colour1[2]))
    });

    println!("Sorted! Selecting colors...");

    for (color, count) in sorted.iter().take(10) {
        println!("{:?} count {}", color, count);
    }

    for (color, count) in sorted.iter().rev().take(10) {
        println!("rev {:?} count {}", color, count);
    }

    let mut sorted_iter = sorted.iter();

    let mut selected_colors: Vec<Rgb<u8>> = Vec::with_capacity(max_colors);
    selected_colors.push(sorted_iter.next().unwrap().0);

    for (key, _value) in sorted_iter {
        if selected_colors.len() < max_colors {
            for selected_color in selected_colors.iter() {
                if rgb_difference(key, selected_color) > rgb_tolerance {
                    selected_colors.push(*key);
                    break;
                }
            }
        } else {
            break;
        }
    }

    for color in selected_colors.iter().take(10) {
        println!("selected {:?}", color);
    }

    println!("Selected {} colors! Creating map...", selected_colors.len());

    let mut color_map: HashMap<Rgb<u8>, Rgb<u8>> = HashMap::with_capacity(sorted.len());
    // Selected colors are themselves
    for color in selected_colors.iter() {
        color_map.insert(*color, *color);
    }

    // Max complexity is O(n * max_colors)
    'sorted_colors: for (key, _value) in sorted.iter() {
        let mut min_difference = f64::MAX;
        let mut min_difference_color = *key;

        for index in 0..selected_colors.len() {
            let difference = rgb_difference(key, unsafe { selected_colors.get_unchecked(index) });
            /*if difference == 0.0 {
                continue 'sorted_colors;
            }*/
            if difference < min_difference {
                min_difference = difference;
                min_difference_color = unsafe { *selected_colors.get_unchecked(index) };
            }
        }

        color_map.insert(*key, min_difference_color);
    }

    println!("Mapped! Filling in image...");

    for pixel in image.pixels_mut() {
        pixel.clone_from(color_map.get(pixel).unwrap());
    }

    println!(
        "Filled! Took {}ms. Recounting colors...",
        Instant::now().duration_since(before_algo).as_millis()
    );

    let mut recounted_colors = Vec::with_capacity(max_colors);
    // Recount colors
    for pixel in image.pixels() {
        if !recounted_colors.contains(pixel) {
            println!("Found unique color! Now {}", recounted_colors.len());
            recounted_colors.push(*pixel);
        }
    }

    println!(
        "Aiming for a max of {} colors, got {}",
        max_colors,
        recounted_colors.len()
    );

    image.save(outname).expect("Failed to write out");
    println!(
        "Took {}ms",
        Instant::now().duration_since(before).as_millis()
    );
}

fn rgb_difference(a: &Rgb<u8>, z: &Rgb<u8>) -> f64 {
    //((a.0[0] as i16 - b.0[0] as i16).abs() + (a.0[1] as i16 - b.0[1] as i16).abs() +(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
    //(a.0[0] as i16 - b.0[0] as i16).abs().max((a.0[1] as i16 - b.0[1] as i16).abs().max(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
    //(a.0[0] as i16 - b.0[0] as i16).abs().max((a.0[1] as i16 - b.0[1] as i16).abs()).max((a.0[2] as i16 - b.0[2] as i16).abs()) as u16
    /*(((a.0[0] as i32 - b.0[0] as i32) * (a.0[0] as i32 - b.0[0] as i32))
    + ((a.0[1] as i32 - b.0[1] as i32) * (a.0[1] as i32 - b.0[1] as i32))
    + ((a.0[2] as i32 - b.0[2] as i32) * (a.0[2] as i32 - b.0[2] as i32)))
    .abs() as u16*/
    let (a, b, c) = pixel_rgb_to_hsv(a);
    let (d, e, f) = pixel_rgb_to_hsv(z);

    (((c - f) * (c - f)) + ((a - d).abs() / 90.0) + (b - e).abs()) as f64
}

#[warn(clippy::float_cmp)]
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
