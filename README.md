Colorsquash is a colour quantization[^1] crate and algorithm.
At it's core, it sorts the unique colours that appear in an image
and selects the most frequent that are sufficiently different.

To put it more clearly:  
The most frequent colour is always selected and placed into the palette.
If the second most frequent colour is *different enough*, it will be selected
as well. If it's not, it is skipped and the third one is tried. This continues
until it selects the necessary amount of colours.

[^1] (wikipedia: color quantization)[https://en.wikipedia.org/wiki/Color_quantization]

### squash
A CLI tool to quantize colours :D

Currently only takes PNG in the RGB colorspace and outputs indexed PNG.