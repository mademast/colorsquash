use std::collections::HashMap;

use image::{io::Reader as ImageReader};
use image::Rgb;

fn main() {
    let filename = std::env::args().skip(1).next().unwrap();
	// The percent of RGB value difference a color has to surpass to be considere unique
	let tolerance = 0.2;
	let rgb_tolerance = (768.0 * tolerance) as u16;
	let max_colors = 256;

	println!("File is {}", &filename);

	let imageread = ImageReader::open(&filename).expect("Failed to open image!");
	let mut image = imageread.decode().expect("Failed to decode image!").into_rgb8();

	println!("Decoded!");

	let mut colors: HashMap<Rgb<u8>, usize> = HashMap::new();

	for pixel in image.pixels() {
		match colors.get_mut(pixel) {
			None => {
				colors.insert(*pixel, 1);
			},
			Some(n) => {
				*n += 1
			}
		}
	}

	println!("{} has {} colors in it", filename, colors.len());
	println!("Sorting...");

	let mut sorted: Vec<(Rgb<u8>, usize)> = colors.into_iter().collect();
	sorted.sort_by(|a, b| a.1.cmp(&b.1).reverse());

	println!("Sorted!");

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

	let mut color_map: HashMap<Rgb<u8>, Rgb<u8>> = HashMap::with_capacity(sorted.len());
	// Selected colors are themselves
	for color in selected_colors.iter() {
		color_map.insert(*color, *color);
	}

	// Max complexity is O(n * max_colors)
	for (key, _value) in sorted.iter() {
		let mut min_difference = 769; // One greater than the max difference
		let mut min_difference_color = *key;

		for index in 0..selected_colors.len() {
			let difference = rgb_difference(key, unsafe { selected_colors.get_unchecked(index) });
			if difference < min_difference {
				min_difference = difference;
				min_difference_color = unsafe {*selected_colors.get_unchecked(index) };
			}
		}

		color_map.insert(*key, min_difference_color);
	}

	for pixel in image.pixels_mut() {
		pixel.clone_from(color_map.get(pixel).unwrap());
	}

	image.save("out.png").expect("Failed to write out");
}

fn rgb_difference(a: &Rgb<u8>, b: &Rgb<u8>) -> u16 {
	((a.0[0] as i16 - b.0[0] as i16).abs() + (a.0[1] as i16 - b.0[1] as i16).abs() +(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
}