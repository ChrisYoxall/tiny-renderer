# Tiny Renderer

A software rasterizer built from scratch in Rust, translating 3D scenes into 2D raster images without GPUs or graphics
libraries.

## Language

**Framebuffer**:
A 2D rectangular memory grid storing the pixel colors of the rendered scene before export.
_Avoid_: Canvas, image buffer, screen buffer

**Clipping**
The process of discarding primitives or pixels that fall outside the visible boundaries of the framebuffer.
_Avoid_: Clamping, truncation, bounds-checking error

**Pixel**:
An individual picture element at discrete integer coordinates `(x, y)` in the framebuffer.
_Avoid_: Dot, point

**Screen Coordinates**:
The 2D coordinate system of the framebuffer, where origin `(0, 0)` is in the bottom-left corner with `x` increasing
rightward and `y` increasing upward.
_Avoid_: Window space, raster space
