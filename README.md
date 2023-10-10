| 24bit RGB | 256 color Indexed|
| - | - |
| ![a small dog laying on a concrete floor in an industrial building](https://nyble.dev/colorsquash/astro.jpg) | ![the same image in 256 color. there are some visual differences, but the two images look very similar](https://nyble.dev/colorsquash/astro_squash.gif) |

colorsquash is a colour quantization[^1] crate and algorithm.  
At it's core, it sorts the colors of an image by how frequently
they appear, greatest to least. It then goes through those colours
and takes the top N colours that are sufficiently different.

[^1]: [wikipedia: color quantization](https://en.wikipedia.org/wiki/Color_quantization)

### squash
A CLI tool to quantize colours :D

Accepts JPEG and PNG as input and can output indexed PNG and GIF.