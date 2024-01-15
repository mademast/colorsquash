use rand::{prelude::*, seq::index::sample};
use rgb::{RGB, RGB8};

pub struct KMeans {
	samples: Vec<RGB8>,
}

impl KMeans {
	pub fn new(samples: Vec<RGB8>) -> Self {
		Self { samples }
	}
	pub fn get_k_colors(&self, k: usize, max_iter: usize) -> Vec<RGB8> {
		let mut centroids = self.get_centroid_seeds_simple(k);
		for _ in 0..max_iter {
			todo!()
		}
		centroids
			.into_iter()
			.map(|c| RGB8::new(c.r.round() as u8, c.g.round() as u8, c.b.round() as u8))
			.collect()
	}

	/// Uses k-means++ algorithm (https://www.mathworks.com/help/stats/kmeans.html#bueq7aj-5)
	fn get_centroid_seeds_simple(&self, k: usize) -> Vec<RGB<f32>> {
		if k >= self.samples.len() {
			return self.samples.iter().map(|&v| v.into()).collect();
		}

		let mut rng = thread_rng();
		let mut centroids: Vec<RGB<f32>> =
			vec![self.samples[rng.gen_range(0..self.samples.len())].into()];
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
