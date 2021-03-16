use std::{collections::HashMap, env::args, time::Instant};

use image::{io::Reader as ImageReader};
use image::Rgb;

fn main() {
	let before = Instant::now();
    let filename = std::env::args().skip(1).next().unwrap();
	let outname = std::env::args().skip(2).next().unwrap();
	// The percent of RGB value difference a color has to surpass to be considere unique
	let tolerance = 0.3;
	let rgb_tolerance = (256.0 * tolerance) as u16;
	let max_colors = 256;

	println!("File is {}", &filename);

	let imageread = ImageReader::open(&filename).expect("Failed to open image!");
	let mut image = imageread.decode().expect("Failed to decode image!").into_rgb8();

	println!("Decoded!");
	let before_algo = Instant::now();

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

	println!("{} has {} colors in it. Sorting most occuring to least...", filename, colors.len());

	let mut sorted: Vec<(Rgb<u8>, usize)> = colors.into_iter().collect();
	sorted.sort_by(|a, b| a.1.cmp(&b.1).reverse());

	println!("Sorted! Selecting colors...");

	for (color, count) in sorted.iter().take(10) {
		println!("{:?} count {}", color, count);
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
		let mut min_difference = 769; // One greater than the max difference
		let mut min_difference_color = *key;

		for index in 0..selected_colors.len() {
			let difference = rgb_difference(key, unsafe { selected_colors.get_unchecked(index) });
			if difference == 0 {
				continue 'sorted_colors;
			}
			if difference < min_difference {
				min_difference = difference;
				min_difference_color = unsafe {*selected_colors.get_unchecked(index) };
			}
		}

		color_map.insert(*key, min_difference_color);
	}

	println!("Mapped! Filling in image...");

	for pixel in image.pixels_mut() {
		pixel.clone_from(color_map.get(pixel).unwrap());
	}

	println!("Filled! Took {}ms. Recounting colors...", Instant::now().duration_since(before_algo).as_millis());

	let mut recounted_colors = Vec::with_capacity(max_colors);
	// Recount colors
	for pixel in image.pixels() {
		if !recounted_colors.contains(pixel) {
			recounted_colors.push(*pixel);
		}
	}
	
	println!("Aiming for a max of {} colors, got {}", max_colors, recounted_colors.len());

	image.save(outname).expect("Failed to write out");
	println!("Took {}ms", Instant::now().duration_since(before).as_millis());
}

fn rgb_difference(a: &Rgb<u8>, b: &Rgb<u8>) -> u16 {
	//((a.0[0] as i16 - b.0[0] as i16).abs() + (a.0[1] as i16 - b.0[1] as i16).abs() +(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
	//(a.0[0] as i16 - b.0[0] as i16).abs().max((a.0[1] as i16 - b.0[1] as i16).abs().max(a.0[2] as i16 - b.0[2] as i16).abs()) as u16
	(a.0[0] as i16 - b.0[0] as i16).abs().max((a.0[1] as i16 - b.0[1] as i16).abs()).max((a.0[2] as i16 - b.0[2] as i16).abs()) as u16
}

//rd.abs().max(gd.abs().max(bd).abs()) as u16
//Diff0: Rgb([92, 77, 40]) Rgb([92, 77, 50])
//0.max(0.max(10))