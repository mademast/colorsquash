use std::collections::HashMap;

#[cfg(rand)]
use rand::{prelude::*, seq::index::sample};
use rgb::{RGB, RGB8};

pub struct KMeans {
	samples: Vec<RGB8>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct HashableRGBF {
	inner: (u32, u32, u32),
}

impl From<RGB<f32>> for HashableRGBF {
	fn from(value: RGB<f32>) -> Self {
		Self {
			inner: (value.r.to_bits(), value.g.to_bits(), value.b.to_bits()),
		}
	}
}

impl KMeans {
	pub fn new(samples: Vec<RGB8>) -> Self {
		Self { samples }
	}
	pub fn get_k_colors(&self, k: usize, max_iter: usize) -> Vec<RGB8> {
		let mut centroids = self.get_centroid_seeds_simple(k);

		for _ in 0..max_iter {
			let mut clusters: HashMap<HashableRGBF, Vec<RGB8>> = HashMap::new();

			for &sample in &self.samples {
				let closest_centroid = Self::closest_centroid(&centroids, sample.into());
				clusters
					.entry(closest_centroid.into())
					.or_default()
					.push(sample);
			}
			centroids = clusters
				.into_values()
				.map(|members| vector_avg(&members))
				.collect()
		}
		centroids
			.into_iter()
			.map(|c| RGB8::new(c.r.round() as u8, c.g.round() as u8, c.b.round() as u8))
			.collect()
	}

	/// Picks a point at random (if feature rand is enabled) for the first centroid, then iteratively adds the point furthest away from any centroid
	/// A more complex solution is the probabilistic k-means++ algorithm (https://www.mathworks.com/help/stats/kmeans.html#bueq7aj-5)
	fn get_centroid_seeds_simple(&self, k: usize) -> Vec<RGB<f32>> {
		if k >= self.samples.len() {
			return self.samples.iter().map(|&v| v.into()).collect();
		}

		#[cfg(rand)]
		let index = thread_rng().gen_range(0..self.samples.len());
		#[cfg(not(rand))]
		let index = 0; //lol

		let mut centroids: Vec<RGB<f32>> = vec![self.samples[index].into()];
		while centroids.len() < k {
			let next = *self
				.samples
				.iter()
				.max_by(|&&v1, &&v2| {
					let v1_closest_centroid = Self::closest_centroid(&centroids, v1.into());
					let v2_closest_centroid = Self::closest_centroid(&centroids, v2.into());

					vector_diff_2_norm(v1.into(), v1_closest_centroid)
						.partial_cmp(&vector_diff_2_norm(v2.into(), v2_closest_centroid))
						.unwrap()
				})
				.unwrap();
			centroids.push(next.into());
		}
		centroids
	}

	fn closest_centroid(centroids: &[RGB<f32>], v: RGB<f32>) -> RGB<f32> {
		*centroids
			.iter()
			.min_by(|&&c1, &&c2| {
				vector_diff_2_norm(c1, v)
					.partial_cmp(&vector_diff_2_norm(c2, v))
					.unwrap()
			})
			.unwrap()
	}

	#[cfg(rand)]
	fn get_centroid_seeds_random(&self, k: usize) -> Vec<RGB<f32>> {
		if k >= self.samples.len() {
			return self.samples.iter().map(|&v| v.into()).collect();
		}

		sample(&mut thread_rng(), self.samples.len(), k)
			.into_iter()
			.map(|i| self.samples[i].into())
			.collect()
	}
}

fn vector_diff(v1: RGB<f32>, v2: RGB<f32>) -> RGB<f32> {
	RGB::new(v1.r - v2.r, v1.g - v2.g, v1.b - v2.b)
}

fn vector_diff_2_norm(v1: RGB<f32>, v2: RGB<f32>) -> f32 {
	let diff = vector_diff(v1, v2);
	(diff.r.powi(2) + diff.g.powi(2) + diff.b.powi(2)).sqrt()
}

fn vector_sum(acc: RGB<f32>, elem: RGB<f32>) -> RGB<f32> {
	RGB::new(acc.r + elem.r, acc.g + elem.g, acc.b + elem.b)
}

fn vector_avg(vs: &[RGB8]) -> RGB<f32> {
	let summed = vs.iter().fold(RGB::new(0.0, 0.0, 0.0), |acc, elem| {
		vector_sum(acc, (*elem).into())
	});
	RGB::new(
		summed.r / vs.len() as f32,
		summed.g / vs.len() as f32,
		summed.b / vs.len() as f32,
	)
}
