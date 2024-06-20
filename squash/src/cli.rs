use std::cmp::Ordering;

use camino::Utf8PathBuf;
use colorsquash::difference::{self, DiffFn};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub struct Cli {
	pub color_count: u8,
	pub tolerance: Option<f32>,
	pub scale: u8,
	pub selector: Selector,
	pub difference: &'static DiffFn,
	pub input: Utf8PathBuf,
	pub in_type: InType,
	pub output: Utf8PathBuf,
	pub out_type: OutType,
	pub verbose: bool,
}

// It's not a builder, but I think the builder/building name is useful
// here because it's used while not all things are populated.
#[derive(Debug, Default)]
struct BuildingCli {
	pub color_count: Option<u8>,
	pub tolerance: Option<f32>,
	pub scale: Option<u8>,
	pub difference: DifferenceFn,
	pub selector: Selector,
	pub verbose: bool,
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

		let difference = match self.difference {
			DifferenceFn::Rgb => &difference::rgb as &DiffFn,
			DifferenceFn::Redmean => &difference::redmean as &DiffFn,
		};

		Cli {
			color_count: self.color_count.unwrap_or(Self::DEFAULT_COLORS),
			tolerance: self.tolerance,
			selector: self.selector,
			scale: self.scale.unwrap_or(25),
			difference,
			input,
			in_type,
			output,
			out_type,
			verbose: self.verbose,
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

#[derive(Debug, Default)]
pub enum Selector {
	#[default]
	SortSelect,
	Kmeans,
	HighestBits,
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
			Some(("scale", scale)) => match scale.parse::<u8>() {
				Err(_) => {
					eprintln!("scale must be >= 1 and <= 100");
					std::process::exit(1);
				}
				Ok(scale) if (1..=100).contains(&scale) => {
					eprintln!("scale must be >= 1 and <= 100");
					std::process::exit(1);
				}
				Ok(scale) => {
					building.scale = Some(scale);
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
			Some(("selector", sel)) | Some(("sel", sel)) => match sel {
				"sort/select" | "sorsel" => building.selector = Selector::SortSelect,
				"kmeans" => building.selector = Selector::Kmeans,
				"highest-bits" => building.selector = Selector::HighestBits,
				_ => {
					eprintln!("'{sel}' is not recognized as a selector. See help=selectors");
					std::process::exit(1);
				}
			},
			Some(("loud", _)) | Some(("verbose", _)) => {
				building.verbose = true;
			}
			Some(("help", "algorithms")) => print_help_algorithms(),
			Some(("help", "selectors")) => print_help_selectors(),
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
	println!("    scale=<int>");
	println!("        the percent of pixels to consider when selecting the palette");
	println!("        for the image. Whole number 1 to 100, inclusive. [Default 25]\n");
	println!("    difference=<algorithm> | dif=<algorithm>");
	println!("        the color comparison function to use. one of: rgb, redmean");
	println!("        for more details use help=algorithms. [Default rgb]\n");
	println!("    selection=<selector> | sel=<selector>");
	println!("        the algorithm for picking the palette. one of: means, sort/select");
	println!("        for more details use help=selectors. [Default sorsel]\n");
	println!("    tolerance=<float> | tol=<float>");
	println!("        how different colours should be to be added to the palette");
	println!("        only sorsel usese this value.");
	println!("        a number > 0 and <= 100 [Default 3]\n");
	println!("    loud= | verbose=");
	println!("        print information about the image and palette.\n");
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

fn print_help_selectors() -> ! {
	println!("SELECTORS:");
	println!("sorsel:");
	println!("    the original colorsquash algorithm. sorts colors from most to least");
	println!("    frequent and then picks the most frequent colors so long as they are");
	println!("    sufficiently different (configurable with tolerance=)\n");
	println!("kmeans:");
	println!("    uses the kmeans clustering algorithm to select colours.");
	println!("    Ignores tolerance=\n");
	println!("highest-bits:");
	println!("    quantizes the colours by shifting the bits of their components until");
	println!("    they all fit in the palette.");
	println!("    Ignores tolerance=");
	std::process::exit(0)
}

fn print_version() -> ! {
	println!("squash version {VERSION}");
	println!("written by {AUTHORS}");
	std::process::exit(0)
}
