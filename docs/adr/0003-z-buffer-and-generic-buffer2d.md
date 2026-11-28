# Floating-Point Z-Buffer and Generic 2D Buffer Abstraction

We decided to extract a generic row-major 2D grid (`Buffer2D<T>`) to back both `TgaImage` (`Buffer2D<Color>`) and a
dedicated `ZBuffer` (`Buffer2D<f32>`, initialized to `f32::NEG_INFINITY` with larger `z` closer to the viewer),
exporting depth visualizations separately as 8-bit grayscale TGA images. This deliberately deviates from
`haqr.eu/tinyrenderer/z-buffer/`, which uses an 8-bit grayscale `TGAImage` directly as the z-buffer, in order to avoid
256-level depth quantization artifacts and prepare for perspective projection and shadow mapping without a later
rewrite.

## Considered Options

- **8-bit `u8` Grayscale `TGAImage` as Z-Buffer (Tutorial Approach)**: Minimal code and directly writable 
to `zbuffer.tga`, but limits depth resolution to 256 discrete slices—causing z-fighting on close geometry and
requiring a rewrite to `f32` once perspective projection is introduced.
- **Standalone `ZBuffer` (`Vec<f32>`) without `Buffer2D<T>`**: Gives full floating-point depth precision with minimal
churn to `TgaImage`, but duplicates 2D bounds checking, row-major indexing, and buffer management logic across
`TgaImage` and `ZBuffer`.
- **Generic `Buffer2D<T>` Backing `TgaImage` (`Buffer2D<Color>`) and `ZBuffer` (`Buffer2D<f32>`)**: Shares zero-cost,
- monomorphized 2D grid storage and `(x, y)` indexing across both buffers while keeping `TgaImage`
concrete (`Buffer2D<Color>`) to avoid complicating the `Framebuffer` trait.

## Consequences

- `ZBuffer` cannot be written directly to disk as TGA; instead, it provides a conversion helper that auto-ranges finite
depth values into an 8-bit `Format::Grayscale` `TgaImage`.
- `TgaImage` continues to store 4 bytes per pixel (`Color`) in memory even when configured as `Format::Rgb` or
`Format::Grayscale`; typed per-format pixel buffers (`Rgb8`, `Gray8`) remain an optional future optimization.
