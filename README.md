# CGA Artifact Color Decoder
Decodes NTSC composite artifact color from CGA RGBI image data. This is not a full NTSC simulation.

CGA images are encoded into the output of the color generation circuit of the CGA based on the original schematic published in the IBM Personal Computer AT Technical Reference, First Edition (1984). That quasi-composite output is then sampled for luma and chroma signals.

Developed for my PC emulator, Marty: https://github.com/dbalsom/marty

# Composite Conversion

Programs that have a Composite output mode utilizing the CGA's high resolution monochrome mode with a foreground color of bright white are essentially congruent to their composite representations - they can be sampled for artifact color direcctly. This is because the CGA color clock generator for White is tied to +5V, so whenever a white pixel appears on the screen the same is output to the composite port.  However, the story is different when color is involved, such as if the foreground color in high resolution mode is changed, or when games use a 320x200 4 color mode (See screenshot below), or text mode effects intended for composite displays such as the famous PC demo, 8088mph.  Naive sampling of the RGBI image data directly for artifact color will not produce the correct result.

![gallery](https://user-images.githubusercontent.com/7229541/215890834-3b57ca17-862d-4348-8f99-6e87d7d7895e.png)

# Credits and Thanks
Thank you to xot and EMMIR

# Usage

Decode 'war.png' to 'output.png' with specified hue, saturation and luminosity, using fast method

cga_artifact --input .\examples\war.png -h 1.5 -s 1.5 -l 1.0 --method fast 

Decode 'king.png' to 'output.png' with specified hue, saturation and luminosity, using accurate method

cga_artifact --input .\examples\king.png -h 1.5 -s 1.5 -l 1.0 --method accurate 
