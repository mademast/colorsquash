use std::collections::HashMap;

#[cfg(not(feature = "simd-kmeans"))]
use crate::nih_kmeans::KMeans;
#[cfg(feature = "simd-kmeans")]
use kmeans::{KMeans, KMeansConfig};
use rgb::RGB8;

use crate::{
	difference::{self, DiffFn},
	ImageData,
};

pub trait Selector {
	// wanted Into<ImageData> here but rustc got mad about vtable building
	// because we store this as Box<dyn Selector> in Squasher and it's builder
	fn select(&mut self, max_colors: usize, image: ImageData) -> Vec<RGB8>;
}

pub struct SortSelect {
	tolerance: f32,
	difference_fn: Box<DiffFn>,
}

impl Selector for SortSelect {
	/// Pick the colors in the palette from a Vec of colors sorted by number
	/// of times they occur, high to low.
	fn select(&mut self, max_colours: usize, image: ImageData) -> Vec<RGB8> {
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

#[derive(Debug, Default)]
pub struct Kmeans {
	pub max_iter: usize,
}

#[cfg(not(feature = "simd-kmeans"))]
impl Selector for Kmeans {
	fn select(&mut self, max_colors: usize, image: ImageData) -> Vec<RGB8> {
		let ImageData(rgb) = image;

		let kmean = KMeans::new(rgb.to_vec());
		kmean.get_k_colors(max_colors, self.max_iter)
	}
}

#[cfg(feature = "simd-kmeans")]
impl Selector for Kmeans {
	fn select(&mut self, max_colors: usize, image: ImageData) -> Vec<RGB8> {
		use rgb::ComponentBytes;

		let ImageData(rgb) = image;

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
			self.max_iter,
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

pub struct HeuristicSorsel {
	tolerance: f32,
	variance: f32,
	max_attempts: usize,
	difference_fn: Box<DiffFn>,
}

impl Selector for HeuristicSorsel {
	/// Pick the colors in the palette from a Vec of colors sorted by number
	/// of times they occur, high to low.
	fn select(&mut self, max_colours: usize, image: ImageData) -> Vec<RGB8> {
		let colors = Self::unique_and_sort(image);

		let mut best = RunData {
			score: f32::MAX,
			palette: vec![],
		};

		let mut attempts = 0;
		//let mut current_tolerance = 9.0 - (max_colours as f32).log2();
		let mut current_tolerance = self.tolerance;
		let mut current_variance = self.variance;

		while attempts < self.max_attempts {
			attempts += 1;

			let higher = current_tolerance + current_variance;
			let lower = current_tolerance - current_variance;

			let run_up = Self::compute_once(&colors, max_colours, higher, &self.difference_fn);
			let run_down = Self::compute_once(&colors, max_colours, lower, &self.difference_fn);

			if run_up.score >= best.score && run_down.score >= best.score {
				// neither was better than the previous best. can we cut the
				// variance to try and fine tune?
				if current_variance > 0.01 {
					// Yes, cut it in half and run again.
					current_variance /= 2.0;
				} else {
					// No, we've reached our limit. Break from the loop
					break;
				}
			} else if run_up.score < run_down.score {
				current_tolerance = higher;
				best = run_up;
			} else {
				current_tolerance = lower;
				best = run_down;
			}
		}

		println!("final tolerance {:.2}", current_tolerance);

		best.palette
	}
}

struct RunData {
	palette: Vec<RGB8>,
	score: f32,
}

impl HeuristicSorsel {
	fn compute_once(
		colors: &[(RGB8, usize)],
		max_colours: usize,
		tolerance: f32,
		diff_fn: &DiffFn,
	) -> RunData {
		let tolerance = (tolerance / 100.0) * 765.0;
		let mut selected_colors: Vec<RGB8> = Vec::with_capacity(max_colours);

		for (sorted_color, _) in colors {
			if max_colours <= selected_colors.len() {
				break;
			} else if selected_colors
				.iter()
				.all(|selected_color| (diff_fn)(selected_color, sorted_color) > tolerance)
			{
				selected_colors.push(*sorted_color);
			}
		}

		// Calculate a score for this tolerance. The total score is the sum of
		// the color scores. The color score is the number of times that colour
		// occures multiplied with the least difference.
		let mut score = 0.0;
		for (color, count) in colors {
			let mut min_diff = f32::MAX;

			for selected in &selected_colors {
				let diff = (diff_fn)(selected, color);
				if diff.max(0.0) < min_diff {
					min_diff = diff;
				}
			}

			score += min_diff * (*count as f32);
		}

		RunData {
			palette: selected_colors,
			score,
		}
	}

	pub fn tolerance(mut self, tolerance: f32) -> Self {
		self.tolerance = tolerance;
		self
	}

	pub fn variance(mut self, vary: f32) -> Self {
		self.variance = vary;
		self
	}

	pub fn max_attempts(mut self, count: usize) -> Self {
		self.max_attempts = count;
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
	fn unique_and_sort<'a, Img>(buffer: Img) -> Vec<(RGB8, usize)>
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

	fn sort(map: HashMap<RGB8, usize>) -> Vec<(RGB8, usize)> {
		let mut sorted: Vec<(RGB8, usize)> = map.into_iter().collect();
		sorted.sort_by(|(colour1, freq1), (colour2, freq2)| {
			freq2
				.cmp(freq1)
				.then(colour2.r.cmp(&colour1.r))
				.then(colour2.g.cmp(&colour1.g))
				.then(colour2.b.cmp(&colour1.b))
		});

		sorted.into_iter().collect()
	}
}

impl Default for HeuristicSorsel {
	fn default() -> Self {
		Self {
			tolerance: 3.0,
			variance: 0.25,
			max_attempts: 10,
			difference_fn: Box::new(difference::rgb),
		}
	}
}
