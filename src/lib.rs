use std::collections::HashMap;
use std::ops::Deref;

use ahash::RandomState;

pub struct Squasher<T> {
    palette: Vec<(Rgb, usize)>,
    larget_count: usize,
    map: Vec<T>,
}

impl<T: Count> Squasher<T> {
    /// Creates a new squasher and allocates a new color map. A color map
    /// contains every 24-bit color and ends up with an amount of memory
    /// equal to `16MB * std::mem::size_of(T)`
    pub fn new(max_colors: T, buffer: &[u8]) -> Self {
        let sorted = Self::unique_and_sort(buffer);
        let selected = Self::select_colors(&sorted, max_colors);

        let mut this = Self {
            palette: selected,
            larget_count: sorted.first().unwrap().1,
            map: vec![T::zero(); 256 * 256 * 256],
        };

        this.map_selected(&sorted);

        this
    }

    /// Take an RGB image buffer and an output buffer. The function will fill
    /// the output buffer with indexes into the Palette.
    pub fn map_image(&mut self, image: &[u8], buffer: &mut [T]) {
        // We have to map the colours of this image now because it might contain
        // colours not present in the first image.
        let sorted = Self::unique_and_sort(image);
        self.map_selected(&sorted);

        for (idx, color) in image.chunks(3).enumerate() {
            let index = self.map[color_index(&Rgb([color[0], color[1], color[2]]))];

            buffer[idx] = index;
        }
    }

    /// Retrieve the palette this squasher is working from
    pub fn palette(&self) -> Vec<Rgb> {
        self.palette.iter().map(|ahh| ahh.0).collect()
    }

    /// Retrieve the palette as bytes
    pub fn palette_bytes(&self) -> Vec<u8> {
        self.palette
            .clone()
            .into_iter()
            .map(|rgb| rgb.0.into_iter())
            .flatten()
            .collect()
    }

    /// Takes an image buffer of RGB data and fill the color map
    fn unique_and_sort(buffer: &[u8]) -> Vec<(Rgb, usize)> {
        let mut colors: HashMap<Rgb, usize, RandomState> = HashMap::default();

        //count pixels
        for pixel in buffer.chunks(3) {
            let rgb = Rgb([pixel[0], pixel[1], pixel[2]]);

            match colors.get_mut(&rgb) {
                None => {
                    colors.insert(rgb, 1);
                }
                Some(n) => *n += 1,
            }
        }

        let mut sorted: Vec<(Rgb, usize)> = colors.into_iter().collect();
        sorted.sort_by(|(colour1, freq1), (colour2, freq2)| {
            freq2
                .cmp(freq1)
                .then(colour2[0].cmp(&colour1[0]))
                .then(colour2[1].cmp(&colour1[1]))
                .then(colour2[2].cmp(&colour1[2]))
        });

        sorted
    }

    fn select_colors(sorted: &[(Rgb, usize)], max_colors: T) -> Vec<(Rgb, usize)> {
        #[allow(non_snake_case)]
        let RGB_TOLERANCE: f32 = 0.04 * 256.0;
        let mut selected_colors: Vec<(Rgb, usize)> = Vec::with_capacity(max_colors.as_usize());

        for (key, count) in sorted.iter() {
            if max_colors.le(&selected_colors.len()) {
                break;
            } else if selected_colors
                .iter()
                .all(|color| rgb_difference(key, &color.0) > RGB_TOLERANCE)
            {
                selected_colors.push((*key, *count));
            }
        }

        selected_colors
    }

    fn map_selected(&mut self, sorted: &[(Rgb, usize)]) {
        for (sorted, _) in sorted {
            let mut min_diff = f32::MAX;
            let mut min_index = usize::MAX;

            for (index, (selected, count)) in self.palette.iter().enumerate() {
                //let count_weight = *count as f32 / self.larget_count as f32;
                let diff = rgb_difference(sorted, selected); // - count_weight * 64.0;

                // This is kind of racist genny
                /*if selected[0] + selected[1] + selected[2] < 72 {
                    continue;
                }*/

                //println!("{diff} - {selected:?}");

                if diff.max(0.0) < min_diff {
                    min_diff = diff;
                    min_index = index;
                }
            }

            self.map[color_index(sorted)] = T::from_usize(min_index);
        }
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

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Rgb([u8; 3]);

impl Deref for Rgb {
    type Target = [u8; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[inline(always)]
fn color_index(c: &Rgb) -> usize {
    c.0[0] as usize * (256 * 256) + c.0[1] as usize * 256 + c.0[2] as usize
}

#[allow(clippy::many_single_char_names)]
#[inline(always)]
fn rgb_difference(a: &Rgb, b: &Rgb) -> f32 {
    let absdiff = |a: u8, b: u8| (a as f32 - b as f32).abs();

    /*let hsv1 = pixel_rgb_to_hsv(a);
    let hsv2 = pixel_rgb_to_hsv(b);*/

    //let diff_max = 3.0;

    absdiff(a.0[0], b.0[0]) + absdiff(a.0[1], b.0[1]) + absdiff(a.0[2], b.0[2])
    /*(((hsv1.0 / 90.0) - (hsv2.0 / 90.0)).abs()
    + (hsv1.1 - hsv2.1).abs()
    + ((hsv1.2 - hsv1.2).abs()))
    / diff_max*/
}

fn pixel_rgb_to_hsv(a: &Rgb) -> (f32, f32, f32) {
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
