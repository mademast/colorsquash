use std::collections::HashMap;

#[cfg(feature = "kmeans")]
use kmeans::{KMeans, KMeansConfig};
use rgb::{ComponentBytes, RGB8};

use crate::{
	difference::{self, DiffFn},
	ImageData,
};

pub trait Selector {
	fn select<'a>(&mut self, max_colors: usize, image: ImageData<'a>) -> Vec<RGB8>;
}

pub struct SortSelect {
	tolerance: f32,
	difference_fn: Box<DiffFn>,
}

impl Selector for SortSelect {
	/// Pick the colors in the palette from a Vec of colors sorted by number
	/// of times they occur, high to low.
	fn select<'a>(&mut self, max_colours: usize, image: ImageData<'a>) -> Vec<RGB8> {
		let sorted = Self::unique_and_sort(image);
		let tolerance = (self.tolerance / 100.0) * 765.0;
		let mut selected_colors: Vec<RGB8> = Vec::with_capacity(max_colours);

		for sorted_color in sorted {
			if max_colours <= selected_colors.len() {
				break;
			} else if selected_colors.iter().all(|selected_color| {
				(self.difference_fn)(selected_color, &sorted_color) > tolerance
			}) {
				selected_colors.push(sorted_color);
			}
		}

		selected_colors
	}
}

impl SortSelect {
	/// How different colours have to be to enter the palette. Should be between
	/// 0.0 and 100.0, but is unchecked.
	pub fn tolerance(mut self, percent: f32) -> Self {
		self.tolerance = percent;
		self
	}

	/// The function to use to compare colours while selecting the palette.
	///
	/// see the [difference] module for functions included with the crate and
	/// information on implementing your own.
	pub fn difference(mut self, diff_fn: &'static DiffFn) -> Self {
		self.difference_fn = Box::new(diff_fn);
		self
	}

	/// Takes an image buffer of RGB data and fill the color map
	fn unique_and_sort<'a, Img>(buffer: Img) -> Vec<RGB8>
	where
		Img: Into<ImageData<'a>>,
	{
		let ImageData(rgb) = buffer.into();
		let mut colors: HashMap<RGB8, usize> = HashMap::default();

		//count pixels
		for px in rgb {
			match colors.get_mut(px) {
				None => {
					colors.insert(*px, 1);
				}
				Some(n) => *n += 1,
			}
		}

		Self::sort(colors)
	}

	fn sort(map: HashMap<RGB8, usize>) -> Vec<RGB8> {
		let mut sorted: Vec<(RGB8, usize)> = map.into_iter().collect();
		sorted.sort_by(|(colour1, freq1), (colour2, freq2)| {
			freq2
				.cmp(freq1)
				.then(colour2.r.cmp(&colour1.r))
				.then(colour2.g.cmp(&colour1.g))
				.then(colour2.b.cmp(&colour1.b))
		});

		sorted.into_iter().map(|(color, _count)| color).collect()
	}
}

impl Default for SortSelect {
	fn default() -> Self {
		Self {
			tolerance: 3.0,
			difference_fn: Box::new(difference::rgb),
		}
	}
}

#[cfg(feature = "kmeans")]
#[derive(Debug, Default)]
pub struct Kmeans;

#[cfg(feature = "kmeans")]
impl Selector for Kmeans {
	fn select<'a>(&mut self, max_colors: usize, image: ImageData<'a>) -> Vec<RGB8> {
		let ImageData(rgb) = image.into();

		let kmean = KMeans::new(
			rgb.as_bytes()
				.iter()
				.map(|u| *u as f32)
				.collect::<Vec<f32>>(),
			rgb.as_bytes().len() / 3,
			3,
		);

		let result = kmean.kmeans_lloyd(
			max_colors,
			100,
			KMeans::init_kmeanplusplus,
			&KMeansConfig::default(),
		);

		result
			.centroids
			.chunks_exact(3)
			.map(|rgb| {
				RGB8::new(
					rgb[0].round() as u8,
					rgb[1].round() as u8,
					rgb[2].round() as u8,
				)
			})
			.collect()
	}
}
