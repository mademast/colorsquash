| 24bit RGB | 256 color Indexed|
| - | - |
| ![a small dog laying on a concrete floor in an industrial building](https://dreamy.place/things/colorsquash/astro.jpg) | ![the same image in 256 color. there are some visual differences, but the two images look very similar](https://dreamy.place/things/colorsquash/astro_squash.gif) |

> [!IMPORTANT]
> This crate isn't *quite* ready yet. There's some tuning and research to be done while selecting the palette. To get the best results you'll likely have to do some trial and error with the tolerance for your specific application.

colorsquash is a colour quantization[^1] crate and algorithm.

At it's core, it sorts the colors of an image by how frequently
they appear, greatest to least. It then goes through those colours
and takes the top N colours that are sufficiently different.

[^1]: [wikipedia: color quantization](https://en.wikipedia.org/wiki/Color_quantization)

**library features**

**`gifed`** - adds the `Squasher::palette_gifed()` method allowing you to
directly get a gifed's Palette struct.

**colour selection algorithms**

*`Sorsel`* - sorts colors most to least frequent and then picks the top colours that are different enough than the colours already picked.  
*`KMeans`* - use k-means clustering to select the palette.  
*`HighestBits`* - bit-shift the color components until all of the colors fit in the palette.

### squash
A CLI tool to quantize colours :D

Accepts JPEG and PNG as input and can output indexed PNG and GIF.
