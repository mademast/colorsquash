use rgb::{ComponentBytes, FromSlice, RGB8};
use std::collections::HashMap;

pub mod difference;

type DiffFn = dyn Fn(&RGB8, &RGB8) -> f32;

pub struct SquasherBuilder<T> {
	max_colours: T,
	difference_fn: Box<DiffFn>,
	tolerance: f32,
}

impl<T: Count> SquasherBuilder<T> {
	pub fn new() -> Self {
		Self::default()
	}

	/// The max number of colors selected for the palette, minus one.
	///
	/// `max_colors(255)` will attempt to make a 256 color palette
	pub fn max_colors(mut self, max_minus_one: T) -> SquasherBuilder<T> {
		self.max_colours = max_minus_one;
		self
	}

	/// The function to use to compare colours.
	///
	/// see the [difference] module for functions included with the crate.
	pub fn difference(mut self, difference: &'static DiffFn) -> SquasherBuilder<T> {
		self.difference_fn = Box::new(difference);
		self
	}

	/// Percent colours have to differ by to be included into the palette.
	/// between and including 0.0 to 100.0
	pub fn tolerance(mut self, percent: f32) -> SquasherBuilder<T> {
		self.tolerance = percent;
		self
	}

	pub fn build<'a, Img>(self, image: Img) -> Squasher<T>
	where
		Img: Into<ImageData<'a>>,
	{
		let mut squasher =
			Squasher::from_parts(self.max_colours, self.difference_fn, self.tolerance);
		squasher.recolor(image);

		squasher
	}
}

impl<T: Count> Default for SquasherBuilder<T> {
	fn default() -> Self {
		Self {
			max_colours: T::from_usize(255),
			difference_fn: Box::new(difference::rgb_difference),
			tolerance: 1.0,
		}
	}
}

pub struct Squasher<T> {
	// one less than the max colours as you can't have a zero colour image.
	max_colours_min1: T,
	palette: Vec<RGB8>,
	map: Vec<T>,
	difference_fn: Box<DiffFn>,
	tolerance_percent: f32,
}

impl<T: Count> Squasher<T> {
	/// Creates a new squasher and allocates a new color map. A color map
	/// contains every 24-bit color and ends up with an amount of memory
	/// equal to `16MB * std::mem::size_of(T)`.
	pub fn new<'a, Img>(max_colors_minus_one: T, buffer: Img) -> Self
	where
		Img: Into<ImageData<'a>>,
	{
		let mut this = Self::from_parts(
			max_colors_minus_one,
			Box::new(difference::rgb_difference),
			1.0,
		);
		this.recolor(buffer);

		this
	}

	pub fn builder() -> SquasherBuilder<T> {
		SquasherBuilder::new()
	}

	pub fn set_tolerance(&mut self, percent: f32) {
		self.tolerance_percent = percent;
	}

	/// Create a new palette from the colours in the given image.
	pub fn recolor<'a, Img>(&mut self, image: Img)
	where
		Img: Into<ImageData<'a>>,
	{
		let sorted = Self::unique_and_sort(image);
		let selected = self.select_colors(sorted);
		self.palette = selected;
	}

	/// Create a Squasher from parts. Noteably, this leave your palette empty
	fn from_parts(max_colours_min1: T, difference_fn: Box<DiffFn>, tolerance: f32) -> Self {
		Self {
			max_colours_min1,
			palette: vec![],
			map: vec![T::zero(); 256 * 256 * 256],
			difference_fn,
			tolerance_percent: tolerance,
		}
	}

	/// Take an RGB image buffer and an output buffer. The function will fill
	/// the output buffer with indexes into the Palette. The output buffer should
	/// be a third of the size of the image buffer.
	pub fn map<'a, Img>(&mut self, image: Img, buffer: &mut [T])
	where
		Img: Into<ImageData<'a>>,
	{
		let ImageData(rgb) = image.into();

		if buffer.len() * 3 < rgb.len() {
			panic!("output buffer too small to fit indexed image");
		}

		// We have to map the colours of this image now because it might contain
		// colours not present in the first image.
		let sorted = Self::unique_and_sort(rgb);
		self.map_selected(&sorted);

		for (idx, color) in rgb.iter().enumerate() {
			buffer[idx] = self.map[color_index(color)];
		}
	}

	/// Like [Squasher::map] but it doesn't recount the input image. This will
	/// cause colors the Squasher hasn't seen before to come out as index 0 which
	/// may be incorrect!
	//TODO: gen- Better name?
	pub fn map_no_recolor<'a, Img>(&self, image: Img, buffer: &mut [T])
	where
		Img: Into<ImageData<'a>>,
	{
		let ImageData(rgb) = image.into();

		if buffer.len() * 3 < rgb.len() {
			panic!("output buffer too small to fit indexed image");
		}

		for (idx, color) in rgb.iter().enumerate() {
			buffer[idx] = self.map[color_index(color)];
		}
	}

	#[cfg(feature = "gifed")]
	pub fn palette_gifed(&self) -> gifed::block::Palette {
		self.palette.as_slice().as_bytes().try_into().unwrap()
	}

	/// Retrieve the palette this squasher is working from
	pub fn palette(&self) -> &[RGB8] {
		&self.palette
	}

	/// Retrieve the palette as bytes
	pub fn palette_bytes(&self) -> Vec<u8> {
		self.palette.as_bytes().to_owned()
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

	/// Pick the colors in the palette from a Vec of colors sorted by number
	/// of times they occur, high to low.
	fn select_colors(&self, sorted: Vec<RGB8>) -> Vec<RGB8> {
		// I made these numbers up
		#[allow(non_snake_case)]
		//let RGB_TOLERANCE: f32 = 0.01 * 765.0;
		//let RGB_TOLERANCE: f32 = 36.0;
		let tolerance = (self.tolerance_percent / 100.0) * 765.0;
		let max_colours = self.max_colours_min1.as_usize() + 1;
		let mut selected_colors: Vec<RGB8> = Vec::with_capacity(max_colours);

		for sorted_color in sorted {
			if max_colours <= selected_colors.len() {
				break;
			} else if selected_colors
				.iter()
				.all(|color| (self.difference_fn)(&sorted_color, color) > tolerance)
			{
				selected_colors.push(sorted_color);
			}
		}

		selected_colors
	}

	/// Pick the closest colour in the palette for each unique color in the image
	fn map_selected(&mut self, sorted: &[RGB8]) {
		for colour in sorted {
			let mut min_diff = f32::MAX;
			let mut min_index = usize::MAX;

			for (index, selected) in self.palette.iter().enumerate() {
				let diff = (self.difference_fn)(colour, selected);

				if diff.max(0.0) < min_diff {
					min_diff = diff;
					min_index = index;
				}
			}

			self.map[color_index(colour)] = T::from_usize(min_index);
		}
	}
}

impl Squasher<u8> {
	/// Takes an RGB image buffer and writes the indicies to the first third of
	/// that buffer. The buffer is not resized.
	///
	/// # Returns
	/// The new size of the image
	pub fn map_over(&mut self, image: &mut [u8]) -> usize {
		// "redundant slicing" here is to drop the mut on the reference because
		// ImageData doesn't have a From<&mut [u8]> and I don't particularly want
		// it to
		#[allow(clippy::redundant_slicing)]
		let sorted = Self::unique_and_sort(&image[..]);

		// We have to map the colours of this image now because it might contain
		// colours not present in the first image.
		self.map_selected(&sorted);

		for idx in 0..(image.len() / 3) {
			let rgb_idx = idx * 3;
			let color = RGB8::new(image[rgb_idx], image[rgb_idx + 1], image[rgb_idx + 2]);
			let color_index = self.map[color_index(&color)];

			image[idx] = color_index;
		}

		image.len() / 3
	}
}

pub trait Count: Copy + Clone {
	fn zero() -> Self;
	fn as_usize(&self) -> usize;
	fn from_usize(from: usize) -> Self;
	fn le(&self, rhs: &usize) -> bool;
}

macro_rules! count_impl {
	($kind:ty) => {
		impl Count for $kind {
			fn zero() -> Self {
				0
			}

			fn as_usize(&self) -> usize {
				*self as usize
			}

			#[inline(always)]
			fn from_usize(from: usize) -> Self {
				from as Self
			}

			#[inline(always)]
			fn le(&self, rhs: &usize) -> bool {
				*self as usize <= *rhs
			}
		}
	};
}

count_impl!(u8);
count_impl!(u16);
count_impl!(u32);
count_impl!(u64);
count_impl!(usize);

pub struct ImageData<'a>(&'a [RGB8]);

impl<'a> From<&'a Vec<u8>> for ImageData<'a> {
	fn from(plain: &'a Vec<u8>) -> Self {
		ImageData(plain.as_rgb())
	}
}

impl<'a> From<&'a [u8]> for ImageData<'a> {
	fn from(plain: &'a [u8]) -> Self {
		ImageData(plain.as_rgb())
	}
}

impl<'a> From<&'a [RGB8]> for ImageData<'a> {
	fn from(rgb: &'a [RGB8]) -> Self {
		ImageData(rgb)
	}
}

#[inline(always)]
fn color_index(c: &RGB8) -> usize {
	c.r as usize * (256 * 256) + c.g as usize * 256 + c.b as usize
}
