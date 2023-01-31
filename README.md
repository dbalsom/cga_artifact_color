# CGA Artifact Color Decoder
Decodes NTSC composite artifact color from CGA RGBI image data. This is not a full NTSC simulation.

CGA images are encoded into the output of the color generation circuit of the CGA based on the original schematic published in the IBM Personal Computer AT Technical Reference, First Edition (1984). That quasi-composite output is then sampled for luma and chroma signals.

Developed for my PC emulator, Marty: https://github.com/dbalsom/marty

![gallery](https://user-images.githubusercontent.com/7229541/215890834-3b57ca17-862d-4348-8f99-6e87d7d7895e.png)

# Credits and Thanks
Thank you to xot and EMMIR

# Usage

Decode 'war.png' to 'output.png' with specified hue, saturation and luminosity, using fast method
cga_artifact --input .\examples\war.png -h 1.5 -s 1.5 -l 1.0 --method fast 

Decode 'king.png' to 'output.png' with specified hue, saturation and luminosity, using accurate method
cga_artifact --input .\examples\king.png -h 1.5 -s 1.5 -l 1.0 --method accurate 
