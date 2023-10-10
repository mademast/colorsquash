use colorsquash::Squasher;

use crate::cli::{InType, OutType};

mod cli;
mod image;

fn main() -> Result<(), anyhow::Error> {
	//gen: I should use clap or at least getopt, but this is fine.
	let cli = cli::get();

	let mut image = match cli.in_type {
		InType::Png => image::get_png(cli.input)?,
		InType::Jpeg => image::get_jpg(cli.input)?,
	};

	let mut squasher = Squasher::new(cli.color_count, &image.data);
	let size = squasher.map_over(&mut image.data);
	image.data.resize(size, 0);

	println!(
		"selected {} colours of max {}",
		squasher.palette().len(),
		cli.color_count
	);

	match cli.out_type {
		OutType::Png => image::save_png(image, squasher, cli.output),
		OutType::Gif => image::save_gif(image, squasher, cli.output),
	}
}
