use camino::Utf8PathBuf;

pub struct Cli {
	pub color_count: u8,
	pub input: Utf8PathBuf,
	pub in_type: InType,
	pub output: Utf8PathBuf,
	pub out_type: OutType,
}

pub enum InType {
	Jpeg,
	Png,
}

pub enum OutType {
	Png,
	Gif,
}

// Get's the CLI arguments or dies trying
pub fn get() -> Cli {
	let usage = || -> ! {
		println!("usage: squash <color count> <input> <output>");
		std::process::exit(0);
	};
	let mut argv = std::env::args().skip(1);

	let color_count: u8 = if let Some(Ok(count)) = argv.next().map(|r| r.parse::<usize>()) {
		if count > 256 {
			eprintln!("max colour count must be 256 or below");
			std::process::exit(1);
		} else {
			(count - 1) as u8
		}
	} else {
		usage()
	};

	let input: Utf8PathBuf = if let Some(path) = argv.next() {
		path.into()
	} else {
		usage();
	};

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

	let output: Utf8PathBuf = if let Some(path) = argv.next() {
		path.into()
	} else {
		usage();
	};

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
		color_count,
		input,
		in_type,
		output,
		out_type,
	}
}
