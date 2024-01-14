use std::collections::HashSet;

#[cfg(kmeans)]
use kmeans::{KMeans, KMeansConfig};
use rgb::{ComponentBytes, FromSlice, RGB8};

pub mod difference;
pub mod selection;

use difference::DiffFn;
use selection::Selector;

pub struct SquasherBuilder<T: Count> {
	max_colours: T,
	difference_fn: Box<DiffFn>,
	selector: Option<Box<dyn Selector + 'static>>,
}

impl<T: Count> SquasherBuilder<T> {
	// I don't want a default here because, to me anyway, Default implies a
	// working struct and this would panic build()
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			max_colours: T::zero(),
			difference_fn: Box::new(difference::rgb),
			selector: None,
		}
	}

	/// The max number of colors selected for the palette, minus one.
	///
	/// `max_colors(255)` will attempt to make a 256 color palette
	pub fn max_colors(mut self, max_minus_one: T) -> Self {
		self.max_colours = max_minus_one;
		self
	}

	/// The function to use to compare colours while mapping the image.
	///
	/// see the [difference] module for functions included with the crate and
	/// information on implementing your own.
	pub fn mapper_difference(mut self, difference: &'static DiffFn) -> Self {
		self.difference_fn = Box::new(difference);
		self
	}

	pub fn selector(mut self, selector: impl Selector + 'static) -> Self {
		self.selector = Some(Box::new(selector));
		self
	}

	pub fn build<'a, Img>(self, image: Img) -> Squasher<T>
	where
		Img: Into<ImageData<'a>>,
	{
		let mut squasher =
			Squasher::from_parts(self.max_colours, self.difference_fn, self.selector.unwrap());
		squasher.recolor(image);

		squasher
	}
}

pub struct Squasher<T> {
	// one less than the max colours as you can't have a zero colour image.
	max_colours_min1: T,
	palette: Vec<RGB8>,
	map: Vec<T>,
	selector: Box<dyn Selector + 'static>,
	difference_fn: Box<DiffFn>,
}

impl<T: Count> Squasher<T> {
	/// Creates a new squasher and allocates a new color map. A color map
	/// contains every 24-bit color and ends up with an amount of memory
	/// equal to `16MB * std::mem::size_of(T)`.
	pub fn new<'a, Img>(
		max_colors_minus_one: T,
		selector: impl Selector + 'static,
		buffer: Img,
	) -> Self
	where
		Img: Into<ImageData<'a>>,
	{
		let mut this = Self::from_parts(
			max_colors_minus_one,
			Box::new(difference::rgb),
			Box::new(selector),
		);
		this.recolor(buffer);

		this
	}

	pub fn builder() -> SquasherBuilder<T> {
		SquasherBuilder::new()
	}

	/// Create a new palette from the colours in the given image.
	pub fn recolor<'a, Img>(&mut self, image: Img)
	where
		Img: Into<ImageData<'a>>,
	{
		self.palette = self
			.selector
			.select(self.max_colours_min1.as_usize() + 1, image.into());
	}

	/// Create a Squasher from parts. Noteably, this leave your palette empty
	fn from_parts(
		max_colours_min1: T,
		difference_fn: Box<DiffFn>,
		selector: Box<dyn Selector>,
	) -> Self {
		Self {
			max_colours_min1,
			palette: vec![],
			map: vec![T::zero(); 256 * 256 * 256],
			difference_fn,
			selector,
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
		let unique = Self::unique_colors(rgb);
		self.map_selected(&unique);

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

	/// Pick the closest colour in the palette for each unique color in the image
	fn map_selected(&mut self, unique: &[RGB8]) {
		for colour in unique {
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

	fn unique_colors(image: &[RGB8]) -> Vec<RGB8> {
		let mut unique: HashSet<RGB8> = HashSet::new();
		for px in image {
			unique.insert(*px);
		}
		unique.into_iter().collect()
	}
}

impl Squasher<u8> {
	/// Takes an RGB image buffer and writes the indicies to the first third of
	/// that buffer. The buffer is not resized.
	///
	/// # Returns
	/// The new size of the image
	pub fn map_over(&mut self, image: &mut [u8]) -> usize {
		// We have to map the colours of this image now because it might contain
		// colours not present in the first image.
		let unique = Self::unique_colors(image.as_rgb());
		self.map_selected(&unique);

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

/// Compute the color index into the big-map-of-all-colours.
#[inline(always)]
fn color_index(c: &RGB8) -> usize {
	c.r as usize * (256 * 256) + c.g as usize * 256 + c.b as usize
}
