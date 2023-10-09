use rgb::RGB8;
use std::collections::HashMap;

type DiffFn = dyn Fn(&RGB8, &RGB8) -> f32;

pub struct Squasher<T> {
    palette: Vec<RGB8>,
    map: Vec<T>,
    difference_fn: Box<DiffFn>,
}

impl<T: Count> Squasher<T> {
    /// Creates a new squasher and allocates a new color map. A color map
    /// contains every 24-bit color and ends up with an amount of memory
    /// equal to `16MB * std::mem::size_of(T)`
    pub fn new(max_colors: T, buffer: &[u8]) -> Self {
        let sorted = Self::unique_and_sort(buffer);
        Self::from_sorted(max_colors, sorted, Box::new(rgb_difference))
    }

    /// Like [Squasher::new] but lets you pass your own difference function
    /// to compare values while selecting colours. The default difference
    /// function sums to difference between the RGB channels.
    pub fn new_with_difference(
        max_colors: T,
        buffer: &[u8],
        difference_fn: &'static DiffFn,
    ) -> Self {
        let sorted = Self::unique_and_sort(buffer);
        Self::from_sorted(max_colors, sorted, Box::new(difference_fn))
    }

    fn from_sorted(max_colors: T, sorted: Vec<(RGB8, usize)>, difference_fn: Box<DiffFn>) -> Self {
        let selected = Self::select_colors(&sorted, max_colors, difference_fn.as_ref());

        let mut this = Self {
            palette: selected,
            map: vec![T::zero(); 256 * 256 * 256],
            difference_fn,
        };

        this.map_selected(&sorted);

        this
    }

    /// Take an RGB image buffer and an output buffer. The function will fill
    /// the output buffer with indexes into the Palette. The output buffer should
    /// be a third of the size of the image buffer.
    pub fn map(&mut self, image: &[u8], buffer: &mut [T]) {
        if buffer.len() * 3 < image.len() {
            panic!("outout buffer too small to fit indexed image");
        }

        // We have to map the colours of this image now because it might contain
        // colours not present in the first image.
        let sorted = Self::unique_and_sort(image);
        self.map_selected(&sorted);

        for (idx, color) in image.chunks(3).enumerate() {
            let index = self.map[color_index(&RGB8::new(color[0], color[1], color[2]))];

            buffer[idx] = index;
        }
    }

    /// Like [Squasher::map] but it doesn't recount the input image. This will
    /// cause colors the Squasher hasn't seen before to come out as index 0 which
    /// may be incorrect.
    //TODO: gen- Better name?
    pub fn map_unsafe(&self, image: &[u8], buffer: &mut [T]) {
        if buffer.len() * 3 < image.len() {
            panic!("outout buffer too small to fit indexed image");
        }

        for (idx, color) in image.chunks(3).enumerate() {
            let index = self.map[color_index(&RGB8::new(color[0], color[1], color[2]))];

            buffer[idx] = index;
        }
    }

    /// Retrieve the palette this squasher is working from
    pub fn palette(&self) -> &[RGB8] {
        &self.palette
    }

    /// Retrieve the palette as bytes
    pub fn palette_bytes(&self) -> Vec<u8> {
        self.palette
            .clone()
            .into_iter()
            .flat_map(|rgb| [rgb.r, rgb.g, rgb.b].into_iter())
            .collect()
    }

    /// Takes an image buffer of RGB data and fill the color map
    fn unique_and_sort(buffer: &[u8]) -> Vec<(RGB8, usize)> {
        let mut colors: HashMap<RGB8, usize> = HashMap::default();

        //count pixels
        for pixel in buffer.chunks(3) {
            let rgb = RGB8::new(pixel[0], pixel[1], pixel[2]);

            match colors.get_mut(&rgb) {
                None => {
                    colors.insert(rgb, 1);
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

        sorted
    }

    fn select_colors(sorted: &[(RGB8, usize)], max_colors: T, difference: &DiffFn) -> Vec<RGB8> {
        #[allow(non_snake_case)]
        let RGB_TOLERANCE: f32 = 0.01 * 768.0;
        let mut selected_colors: Vec<(RGB8, usize)> = Vec::with_capacity(max_colors.as_usize());

        for (key, count) in sorted.iter() {
            if max_colors.le(&selected_colors.len()) {
                break;
            } else if selected_colors
                .iter()
                .all(|color| difference(key, &color.0) > RGB_TOLERANCE)
            {
                selected_colors.push((*key, *count));
            }
        }

        selected_colors
            .into_iter()
            .map(|(color, _count)| color)
            .collect()
    }

    fn map_selected(&mut self, sorted: &[(RGB8, usize)]) {
        for (sorted, _) in sorted {
            let mut min_diff = f32::MAX;
            let mut min_index = usize::MAX;

            for (index, selected) in self.palette.iter().enumerate() {
                let diff = (self.difference_fn)(sorted, selected);

                if diff.max(0.0) < min_diff {
                    min_diff = diff;
                    min_index = index;
                }
            }

            self.map[color_index(sorted)] = T::from_usize(min_index);
        }
    }
}

impl Squasher<u8> {
    /// Takes an RGB image buffer and writes the indicies to the first third of
    /// that buffer. The buffer is not resized.
    ///
    /// # Returns
    /// The new size of the image
    pub fn map_over(&self, image: &mut [u8]) -> usize {
        for idx in 0..image.len() {
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

#[inline(always)]
fn color_index(c: &RGB8) -> usize {
    c.r as usize * (256 * 256) + c.g as usize * 256 + c.b as usize
}

/// The default comparison function. Returns a sum of the channel differences.
/// I.E. `|a.red - b.red| + |a.green - b.green| + |a.blue - b.blue|`
#[allow(clippy::many_single_char_names)]
#[inline(always)]
pub fn rgb_difference(a: &RGB8, b: &RGB8) -> f32 {
    let absdiff = |a: u8, b: u8| (a as f32 - b as f32).abs();
    absdiff(a.r, b.r) + absdiff(a.g, b.g) + absdiff(a.b, b.b)
}
