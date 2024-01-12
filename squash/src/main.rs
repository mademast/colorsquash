use cli::DifferenceFn;
use colorsquash::SquasherBuilder;

use crate::cli::{InType, OutType};

mod cli;
mod image;

fn main() -> Result<(), anyhow::Error> {
	//gen: I should use clap or at least getopt, but this is fine.
	//gen: I like experimenting with the cli :)
	let cli = cli::build();

	let mut image = match cli.in_type {
		InType::Png => image::get_png(cli.input)?,
		InType::Jpeg => image::get_jpg(cli.input)?,
	};

	let mut builder = SquasherBuilder::default().max_colors(cli.color_count);

	if let Some(tol) = cli.tolerance {
		builder = builder.tolerance(tol);
	}

	builder = match cli.difference {
		DifferenceFn::Rgb => builder.difference(&colorsquash::difference::rgb),
		DifferenceFn::Redmean => builder.difference(&colorsquash::difference::redmean),
	};

	let mut squasher = builder.build(&image.data);

	let size = squasher.map_over(&mut image.data);
	image.data.resize(size, 0);

	match cli.out_type {
		OutType::Png => image::save_png(image, squasher, cli.output),
		OutType::Gif => image::save_gif(image, squasher, cli.output),
	}
}
