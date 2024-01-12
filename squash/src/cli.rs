use std::cmp::Ordering;

use camino::Utf8PathBuf;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub struct Cli {
	pub color_count: u8,
	pub tolerance: Option<f32>,
	pub difference: DifferenceFn,
	pub input: Utf8PathBuf,
	pub in_type: InType,
	pub output: Utf8PathBuf,
	pub out_type: OutType,
}

// It's not a builder, but I think the builder/building name is useful
// here because it's used while not all things are populated.
#[derive(Debug, Default)]
struct BuildingCli {
	pub color_count: Option<u8>,
	pub tolerance: Option<f32>,
	pub difference: DifferenceFn,
}

impl BuildingCli {
	// One minus max
	const DEFAULT_COLORS: u8 = 255;

	pub fn build_or_die(self, input: &str, output: &str) -> Cli {
		let input: Utf8PathBuf = input.into();
		let in_type = match input.extension() {
			None => {
				eprintln!("can't determine input filetype!\nSupported input types: PNG, JPG");
				std::process::exit(1);
			}
			Some("png") => InType::Png,
			Some("jpg") | Some("jpeg") => InType::Jpeg,
			Some(ext) => {
				eprintln!("unknown filetype '{ext}'!\nSupported input types: PNG, JPG");
				std::process::exit(1);
			}
		};

		let output: Utf8PathBuf = output.into();
		let out_type = match output.extension() {
			None => {
				eprintln!("can't determine output filetype!");
				std::process::exit(1);
			}
			Some("png") => OutType::Png,
			Some("gif") => OutType::Gif,
			Some(ext) => {
				eprintln!("unknown filetype '{ext}'!\nSupport output types are: GIF, PNG");
				std::process::exit(1);
			}
		};

		Cli {
			color_count: self.color_count.unwrap_or(Self::DEFAULT_COLORS),
			tolerance: self.tolerance,
			difference: self.difference,
			input,
			in_type,
			output,
			out_type,
		}
	}
}

pub enum InType {
	Jpeg,
	Png,
}

pub enum OutType {
	Png,
	Gif,
}

#[derive(Debug, Default)]
pub enum DifferenceFn {
	#[default]
	Rgb,
	Redmean,
}

pub fn build() -> Cli {
	let mut free = vec![];
	let mut building = BuildingCli::default();

	for arg in std::env::args().skip(1) {
		// Handle the special cases we want to obey.
		// -h/--help are standards and, even though we're playing with a
		// dd-style syntax, we want to respect these.
		// we'll do -V/--version
		if arg == "-h" || arg == "--help" {
			print_help()
		} else if arg == "-V" || arg == "--version" {
			print_version()
		}

		match arg.split_once('=') {
			None => free.push(arg),
			Some(("colors", value)) | Some(("colours", value)) | Some(("clrs", value)) => {
				match value.parse::<usize>() {
					Err(_) => {
						eprintln!("color must be a whole number > 0 and <= 256");
						std::process::exit(1);
					}
					Ok(count) if count == 0 || count > 256 => {
						eprintln!("color must be a whole number >= 1 and <= 256");
						std::process::exit(1);
					}
					Ok(count) => {
						//TODO: error if this's been set already?
						building.color_count = Some((count - 1) as u8);
					}
				}
			}
			Some(("tolerance", value)) | Some(("tol", value)) => match value.parse::<f32>() {
				Err(_) => {
					eprintln!("tolerance must be > 0.0 and <= 100.0");
					std::process::exit(1);
				}
				Ok(tol) if tol <= 0.0 || tol > 100.0 => {
					eprintln!("tolerance must be > 0.0 and <= 100.0");
					std::process::exit(1);
				}
				Ok(tol) => {
					//TODO: error if this's been set already
					building.tolerance = Some(tol);
				}
			},
			Some(("difference", algo)) | Some(("dif", algo)) => match algo {
				"rgb" => building.difference = DifferenceFn::Rgb,
				"redmean" => building.difference = DifferenceFn::Redmean,
				_ => {
					eprintln!("'{algo}' is not recognized as an algorithm. See help=algorithms");
					std::process::exit(1);
				}
			},
			Some(("help", "algorithms")) => print_help_algorithms(),
			Some(("help", _)) => print_help(),
			Some(("version", _)) => print_version(),
			Some((key, _)) => {
				eprintln!("unrecognised key {key}");
				std::process::exit(1);
			}
		}
	}

	match free.len().cmp(&2) {
		Ordering::Less => {
			eprintln!("didn't get enough arguments! 'help=' for help");
			std::process::exit(1);
		}
		Ordering::Greater => {
			eprintln!("got too many arguments! 'help=' for help");
			std::process::exit(1);
		}
		Ordering::Equal => building.build_or_die(&free[0], &free[1]),
	}
}

fn print_help() -> ! {
	println!("usage: {NAME} [arguments ...] <input> <output>\n");
	println!("<input>  path to a jpeg or png file");
	println!("<output> path to write a png or gif file to\n");
	println!("ARGUMENTS:");
	println!("    colors=<int> | clrs=<int>");
	println!("        the number of colours the final image should contain");
	println!("        a whole number more than 0 and less than, or equal, 256");
	println!("        [Default 256]\n");
	println!("    difference=<algorithm> | did=<algorithm>");
	println!("        the color comparison function to use. one of: rgb, redmean");
	println!("        for more details use help=algorithms. [Default rgb]");
	println!("    tolerance=<float> | tol=<float>");
	println!("        how different colours should be to be added to the palette");
	println!("        a number > 0 and <= 100\n");
	println!("    help= | -h | --help");
	println!("        print this message and exit\n");
	println!("    version= | -V | --version");
	println!("        print the version and authors and exit");
	std::process::exit(0)
}

fn print_help_algorithms() -> ! {
	println!("ALGORITHMS");
	println!("rgb:");
	println!("    a straight, rather naïve, RGB comparison. It sums the channel");
	println!("    differences. This is it, really:");
	println!("    |a.red - b.red| + |a.green - b.green| + |a.blue - b.blue|\n");
	println!("redmean:");
	println!("    a slightly more intelligent algorithm that weighs the channels");
	println!("    in an attempt to more better align with human color perception.");
	std::process::exit(0)
}

fn print_version() -> ! {
	println!("squash version {VERSION}");
	println!("written by {AUTHORS}");
	std::process::exit(0)
}
