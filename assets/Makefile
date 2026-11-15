.PHONY: all
all: $(wildcard *.png wildcard *.ico)

apple-touch-icon.png: favicon.svg
	inkscape --export-filename=$@ --export-width=180 $^

favicon-512x512.png: favicon.svg
	inkscape --export-filename=$@ --export-width=512 $^

favicon-192x192.png: favicon.svg
	inkscape --export-filename=$@ --export-width=192 $^

favicon-32x32.png: favicon.svg
	inkscape --export-filename=$@ --export-width=32 $^

favicon-16x16.png: favicon.svg
	inkscape --export-filename=$@ --export-width=16 $^

# Why -density 384 matters:
# SVGs have no native pixel dimensions. Setting a high density forces
# ImageMagick to rasterize the vector at a large resolution before scaling
# down, preventing blurry edges.
favicon.ico: favicon.svg
	magick -background none -density 384 $^ -define icon:auto-resize=256,128,64,48,32,16 $@
