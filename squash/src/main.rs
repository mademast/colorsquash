use std::{fs::File, io::BufWriter};

use anyhow::bail;
use camino::{Utf8Path, Utf8PathBuf};
use colorsquash::Squasher;
use png::{ColorType, Decoder, Encoder};

fn main() -> Result<(), anyhow::Error> {
	// I should use clap or at least getopt, but this is fine. It's 20LOC.
	let usage = || -> ! {
		println!("usage: squash <color count> <input> <output>");
		std::process::exit(0);
	};
	let mut argv = std::env::args().skip(1);

	let color_count: u8 = if let Some(Ok(count)) = argv.next().map(|r| r.parse()) {
		count
	} else {
		usage()
	};

	let input_path: Utf8PathBuf = if let Some(path) = argv.next() {
		path.into()
	} else {
		usage();
	};

	let output_path: Utf8PathBuf = if let Some(path) = argv.next() {
		path.into()
	} else {
		usage();
	};

	let mut image = get_png(input_path)?;

	let squasher = Squasher::new(color_count, &image.data);
	let size = squasher.map_over(&mut image.data);
	image.data.resize(size, 0);

	// PNG Output
	let file = File::create(output_path)?;
	let bufw = BufWriter::new(file);

	let mut enc = Encoder::new(bufw, image.width as u32, image.height as u32);
	enc.set_color(ColorType::Indexed);
	enc.set_depth(png::BitDepth::Eight);
	enc.set_palette(squasher.palette_bytes());
	enc.write_header()?.write_image_data(&image.data)?;

	Ok(())
}

fn get_png<P: AsRef<Utf8Path>>(path: P) -> Result<Image, anyhow::Error> {
	let decoder = Decoder::new(File::open(path.as_ref())?);
	let mut reader = decoder.read_info()?;

	let mut buf = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut buf)?;
	let data = &buf[..info.buffer_size()];

	println!(
		"{}x{} * 3 = {} | out={}, bs={}",
		info.width,
		info.height,
		info.width as usize * info.height as usize * 3,
		buf.len(),
		info.buffer_size()
	);

	let colors = info.color_type;
	match colors {
		ColorType::Grayscale | ColorType::GrayscaleAlpha | ColorType::Indexed | ColorType::Rgba => {
			bail!("colortype {colors:?} not supported")
		}
		ColorType::Rgb => Ok(Image {
			width: info.width as usize,
			height: info.height as usize,
			data: data.to_vec(),
		}),
	}
}

struct Image {
	width: usize,
	height: usize,
	data: Vec<u8>,
}
