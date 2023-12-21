## A Makefile to compile SPIR-V shaders.
## Maybe it should be replaced by cargo-make.

.PHONY: shaders

## Relative path to shaders.
SHADERS= ensnano-utils/src/circles2d/circle.vert.spv\
         ensnano-utils/src/circles2d/circle.frag.spv\
         ensnano-utils/src/circles2d/rotation_widget.frag.spv

shaders: $(SHADERS)

%.vert.spv: %.vert
	glslc $< -o $@

%.frag.spv: %.frag
	glslc $< -o $@
