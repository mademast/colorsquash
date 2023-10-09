use std::{fs::File, io::BufWriter};

use anyhow::{anyhow, bail};
use camino::{Utf8Path, Utf8PathBuf};
use colorsquash::Squasher;
use gifed::writer::{GifBuilder, ImageBuilder};
use png::{ColorType, Decoder, Encoder};
use zune_jpeg::{zune_core::colorspace::ColorSpace, JpegDecoder};

fn main() -> Result<(), anyhow::Error> {
	// I should use clap or at least getopt, but this is fine. It's 20LOC.
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

	let mut image = match input_path.extension() {
		None => {
			eprintln!("can't determine input filetype!\nSupported input types: PNG, JPG");
			std::process::exit(1);
		}
		Some("png") => get_png(input_path)?,
		Some("jpg") | Some("jpeg") => get_jpg(input_path)?,
		Some(ext) => {
			eprintln!("unknown filetype '{ext}'!\nSupported input types: PNG, JPG");
			std::process::exit(1);
		}
	};

	let squasher =
		Squasher::new_with_difference(color_count, &image.data, &colorsquash::redmean_difference);
	let size = squasher.map_over(&mut image.data);
	image.data.resize(size, 0);

	println!(
		"selected {} colours of max {}",
		squasher.palette().len(),
		color_count
	);

	match output_path.extension() {
		None => {
			eprintln!("can't determine output filetype! defaulting to png");
			save_png(image, squasher, output_path)
		}
		Some("png") => save_png(image, squasher, output_path),
		Some("gif") => save_gif(image, squasher, output_path),
		Some(ext) => {
			eprintln!("unknown filetype '{ext}'!\nSupport output types are: GIF, PNG");
			std::process::exit(1);
		}
	}
}

struct Image {
	width: usize,
	height: usize,
	data: Vec<u8>,
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

fn get_jpg<P: AsRef<Utf8Path>>(path: P) -> Result<Image, anyhow::Error> {
	let content = std::fs::read(path.as_ref())?;
	let mut dec = JpegDecoder::new(&content);
	let pixels = dec.decode()?;
	let info = dec
		.info()
		.ok_or(anyhow!("image had no info; this should be impossible"))?;

	let colorspace = dec.get_output_colorspace();
	match colorspace {
		Some(ColorSpace::RGB) => (),
		_ => bail!("colorspace {colorspace:?} not supported"),
	}

	Ok(Image {
		width: info.width as usize,
		height: info.height as usize,
		data: pixels,
	})
}

fn save_png(image: Image, squasher: Squasher<u8>, path: Utf8PathBuf) -> Result<(), anyhow::Error> {
	let file = File::create(path)?;
	let bufw = BufWriter::new(file);

	let mut enc = Encoder::new(bufw, image.width as u32, image.height as u32);
	enc.set_color(ColorType::Indexed);
	enc.set_depth(png::BitDepth::Eight);
	enc.set_palette(squasher.palette_bytes());
	enc.write_header()?.write_image_data(&image.data)?;

	Ok(())
}

fn save_gif(image: Image, squasher: Squasher<u8>, path: Utf8PathBuf) -> Result<(), anyhow::Error> {
	// I don't think I like this API anymore. It's a builder API, that's fine.
	// I should make it so you can mutate the Gif directly.
	GifBuilder::new(image.width as u16, image.height as u16)
		.palette(squasher.palette_bytes().as_slice().try_into().unwrap())
		.image(ImageBuilder::new(image.width as u16, image.height as u16).build(image.data)?)
		.build()?
		.save(path)?;

	Ok(())
}
