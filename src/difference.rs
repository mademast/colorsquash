//! A set of difference functions you can use with [SquasherBuilder::difference]

use rgb::RGB8;

/// A naïve comparison just summing the channel differences
/// I.E. `|a.red - b.red| + |a.green - b.green| + |a.blue - b.blue|`
#[allow(clippy::many_single_char_names)]
#[inline(always)]
pub fn rgb_difference(a: &RGB8, b: &RGB8) -> f32 {
    let absdiff = |a: u8, b: u8| (a as f32 - b as f32).abs();
    absdiff(a.r, b.r) + absdiff(a.g, b.g) + absdiff(a.b, b.b)
}

// https://en.wikipedia.org/wiki/Color_difference#sRGB
#[inline(always)]
pub fn redmean_difference(a: &RGB8, b: &RGB8) -> f32 {
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
