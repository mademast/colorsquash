use std::{fs::File, io::BufWriter};

use anyhow::{anyhow, bail};
use camino::{Utf8Path, Utf8PathBuf};
use colorsquash::{selection::SortSelect, Squasher};
use gifed::{writer::ImageBuilder, Gif};
use png::{ColorType, Decoder, Encoder};
use zune_jpeg::{zune_core::colorspace::ColorSpace, JpegDecoder};

pub struct Image {
	pub width: usize,
	pub height: usize,
	pub data: Vec<u8>,
}

pub fn get_png<P: AsRef<Utf8Path>>(path: P) -> Result<Image, anyhow::Error> {
	let decoder = Decoder::new(File::open(path.as_ref())?);
	let mut reader = decoder.read_info()?;

	let mut data = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut data)?;
	data.resize(info.buffer_size(), 0);

	let colors = info.color_type;
	match colors {
		ColorType::Grayscale | ColorType::GrayscaleAlpha | ColorType::Indexed => {
			bail!("colortype {colors:?} not supported")
		}
		ColorType::Rgba => {
			let pixels = info.width as usize * info.height as usize;

			// the first RGB is fine, we don't need to touch it
			for idx in 1..pixels {
				data[idx * 3] = data[idx * 4];
				data[idx * 3 + 1] = data[idx * 4 + 1];
				data[idx * 3 + 2] = data[idx * 4 + 2];
			}
			data.resize(pixels * 3, 0);

			Ok(Image {
				width: info.width as usize,
				height: info.height as usize,
				data,
			})
		}
		ColorType::Rgb => Ok(Image {
			width: info.width as usize,
			height: info.height as usize,
			data,
		}),
	}
}

pub fn get_jpg<P: AsRef<Utf8Path>>(path: P) -> Result<Image, anyhow::Error> {
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

pub fn save_png(
	image: Image,
	squasher: Squasher<u8>,
	path: Utf8PathBuf,
) -> Result<(), anyhow::Error> {
	let file = File::create(path)?;
	let bufw = BufWriter::new(file);

	let mut enc = Encoder::new(bufw, image.width as u32, image.height as u32);
	enc.set_color(ColorType::Indexed);
	enc.set_depth(png::BitDepth::Eight);
	enc.set_palette(squasher.palette_bytes());
	enc.write_header()?.write_image_data(&image.data)?;

	Ok(())
}

pub fn save_gif(
	image: Image,
	squasher: Squasher<u8>,
	path: Utf8PathBuf,
) -> Result<(), anyhow::Error> {
	let mut gif = Gif::new(image.width as u16, image.height as u16);
	gif.set_palette(Some(squasher.palette_gifed()));
	gif.push(ImageBuilder::new(image.width as u16, image.height as u16).build(image.data)?);
	gif.save(path)?;

	Ok(())
}
