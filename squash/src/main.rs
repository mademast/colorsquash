use colorsquash::{
	selection::{Kmeans, SortSelect},
	SquasherBuilder,
};

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

	let mut builder = SquasherBuilder::new()
		.max_colors(cli.color_count)
		.mapper_difference(cli.difference);

	match cli.selector {
		cli::Selector::SortSelect => {
			let mut sorsel = SortSelect::default().difference(cli.difference);
			if let Some(tol) = cli.tolerance {
				sorsel = sorsel.tolerance(tol)
			}

			builder = builder.selector(sorsel);
		}
		cli::Selector::Kmeans => builder = builder.selector(Kmeans { max_iter: 10 }),
	};

	let start = std::time::Instant::now();
	let mut squasher = builder.build(&image.data);
	println!("{:.2}ms", start.elapsed().as_secs_f32());

	let size = squasher.map_over(&mut image.data);
	image.data.resize(size, 0);

	match cli.out_type {
		OutType::Png => image::save_png(image, squasher, cli.output),
		OutType::Gif => image::save_gif(image, squasher, cli.output),
	}
}
