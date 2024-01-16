use std::time::Duration;

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

	let mut start = std::time::Instant::now();
	let mut squasher = builder.scale(cli.scale).build(&image.data);

	if cli.verbose {
		println!(
			"Palette is {} colors.\nSelection took {}",
			squasher.palette().len(),
			human_time(start.elapsed())
		);
	}

	start = std::time::Instant::now();
	let size = squasher.map_over(&mut image.data);

	if cli.verbose {
		println!("Mapping took {}", human_time(start.elapsed()));
	}

	image.data.resize(size, 0);

	match cli.out_type {
		OutType::Png => image::save_png(image, squasher, cli.output),
		OutType::Gif => image::save_gif(image, squasher, cli.output),
	}
}

fn human_time(duration: Duration) -> String {
	if duration.as_secs() > 0 {
		format!("{:.2}s", duration.as_secs_f32())
	} else if duration.as_millis() >= 10 {
		format!("{}ms", duration.as_millis())
	} else {
		format!("{}μs", duration.as_micros())
	}
}
