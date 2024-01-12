//! A set of difference functions you can use with [SquasherBuilder::difference()]
//!
//! # Writing your own difference function
//! The type you want is `dyn Fn(&RGB8, &RGB8) -> f32`  
//! (defined as [`DiffFn`])
//!
//! The first argument is the color already in the palette and the second is
//! the color we're checking. These are [RGB8] which is a rexport from the `rgb`
//! crate.
//!
//! The value returned is between 0 and 768, but that's not a hard-rule. If you
//! return a value out of that range you'll have to adjust the tolerance with
//! [Squasher::set_tolerance()] or [SquasherBuilder::tolerance].
//!
//! The difference functions have the possibility of being called hundreds of
//! thousands of times; you might want to `#[inline(always)]`

// This is used in the module level documentation just above. Without it we'd
// have to fully qualify the interlink which is also how it'd be displayed.
#[allow(unused_imports)]
use crate::{Squasher, SquasherBuilder};

// rexport this so people don't need to add the rgb crate to their project. this
// also helps avoid crate version mismatch
/// rexport from the [`rgb`](https://docs.rs/rgb/0.8.37/rgb/) crate.
pub use rgb::RGB8;

/// Type definition for difference functions.
pub type DiffFn = dyn Fn(&RGB8, &RGB8) -> f32;

/// A naïve comparison just summing the channel differences
/// I.E. `|a.red - b.red| + |a.green - b.green| + |a.blue - b.blue|`
#[allow(clippy::many_single_char_names)]
#[inline(always)]
pub fn rgb(a: &RGB8, b: &RGB8) -> f32 {
	let absdiff = |a: u8, b: u8| (a as f32 - b as f32).abs();
	absdiff(a.r, b.r) + absdiff(a.g, b.g) + absdiff(a.b, b.b)
}

// https://en.wikipedia.org/wiki/Color_difference#sRGB
/// a slightly more intelligent algorithm that weighs the channels in an attempt
/// to better align with human color perception.
#[inline(always)]
pub fn redmean(a: &RGB8, b: &RGB8) -> f32 {
	let delta_r = a.r as f32 - b.r as f32;
	let delta_g = a.g as f32 - b.g as f32;
	let delta_b = a.b as f32 - b.b as f32;
	// reasonably sure calling it prime is wrong, but
	let r_prime = 0.5 * (a.r as f32 + b.r as f32);

	let red_part = (2.0 + (r_prime / 256.0)) * (delta_r * delta_r);
	let green_part = 4.0 * (delta_g * delta_g);
	let blue_part = (2.0 + (255.0 - r_prime) / 256.0) * (delta_b * delta_b);

	(red_part + green_part + blue_part).sqrt()
}
