# Tiny Renderer (Rust)

A pure-Rust reimplementation of Dmitry V. Sokolov's [tiny renderer](https://haqr.eu/tinyrenderer/) course. The source
code is [here](https://github.com/ssloy/tinyrenderer).

Aim is learning & fun, rather than a re-usable production grade graphics library.

Have implemented through hidden-surface removal (z-buffer). Next time start at https://haqr.eu/tinyrenderer/camera/

## Architectural Decisions

Uses the standard library only to learn core principles rather than using `glam`, `nalgebra`, `ndarray` crates for
linear algebra or any image crates.

Uses a bottom-left origin `(0, 0)` to match Cartesian / OpenGL conventions.

Stores mesh faces as contiguous 3-vertex triangle arrays (`Vec<[usize; 3]>`) with load-time fan triangulation (assuming
convex faces) rather than dynamic polygons (`Vec<Vec<usize>>`). This eliminates per-face heap allocations and aligns
with graphics rasterization primitives (see [ADR 0002](docs/adr/0002-triangular-mesh-representation.md)).

Shares a generic row-major `Buffer2D<T>` across both `TgaImage` (`Buffer2D<Color>`) and `ZBuffer` (`Buffer2D<f32>`),
exporting depth visualizations separately as 8-bit grayscale TGA images (see [ADR 0003](docs/adr/0003-z-buffer-and-generic-buffer2d.md)).

## Running

```bash
cargo run
```

Run unit tests:
```bash
cargo test
```

## Items for future consideration

### 1. Viewport Clipping vs. Idiomatic Rust Error Handling

When confirming whether a specific pixel can be set there was a choice on whether to follow the typical Rust convention
and return Result<T, E> or panic (assert!) or to return a boolean indicating that it couldn't be set. The agent
recommended a boolean for performance reasons.

In C++, try / catch has what is called "zero-cost exceptions":
- Happy path (no error): The CPU executes straight-line code with zero branch instructions. 
- Error path (thrown): The runtime pauses, inspects metadata tables, and walks the stack backwards, which is thousands
of times slower.

In Rust, Result<T, E> is just an enum (a tagged union):

    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

Every time the function returns, the CPU has to write the tag (discriminant), and the caller has to check that tag to
see whether it succeeded.

Does this slow down the CPU? If OutOfBoundsError is a unit type (zero-sized, e.g. struct OutOfBounds;), then
`Result<(), OutOfBounds>` consists solely of a 1-byte discriminant (0 for Ok, 1 for Err). At the assembly level, this
occupies the exact same CPU register (al on x86-64) as a bool (true or false).

When the function is inlined by LLVM (`#[inline]` or in release builds with `--release`) the compiler can often eliminate
the tag entirely and merge the bounds check with whatever the caller was doing so the cost is negligible/non-existant.
However, in debug mode (cargo build / cargo run without optimization), inlining is disabled. In software rendering
where you might draw 1,000,000+ pixels per frame, calling a non-inlined function that creates and matches enums in an
inner loop will noticeably degrade framerates.

This is optimised away in later lessons by checking beforehand rather than when the pixel is set.

### 2. Typed Per-Format Pixel Buffers (`Rgb8` / `Gray8`)

In Lesson 3, the generic `Buffer2D<T>` abstraction was introduced to back both `TgaImage` (`Buffer2D<Color>`) and `ZBuffer` (`Buffer2D<f32>`)—see [ADR 0003](docs/adr/0003-z-buffer-and-generic-buffer2d.md). `TgaImage` still stores 4 bytes per pixel (`Color`) in memory even for `Format::Rgb` and `Format::Grayscale` images; introducing strongly typed per-format pixel buffers (`Rgb8([u8; 3])`, `Rgba8([u8; 4])`, `Gray8(u8)`) remains a future option if memory footprint becomes a priority.


