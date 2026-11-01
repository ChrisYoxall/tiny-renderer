# Tiny Renderer (Rust)

A pure-Rust reimplementation of Dmitry V. Sokolov's [tiny renderer](https://haqr.eu/tinyrenderer/) course. The source
code is [here](https://github.com/ssloy/tinyrenderer).

Aim is learning & fun, rather than a re-usable production grade graphics library.

## Architectural Decisions

Uses the standard library only to learn core principles rather than using `glam`, `nalgebra`, `ndarray` crates for
linear algebra or any image crates.

Uses a bottom-left origin `(0, 0)` to match Cartesian / OpenGL conventions.

## Running

Run the Lesson 0 program to generate `framebuffer.tga`:
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

When the function is inlined by LLVM (#[inline] or in release builds with --release) the compiler can often eliminate
the tag entirely and merge the bounds check with whatever the caller was doing so the cost is negligible/non-existant.
However, in debug mode (cargo build / cargo run without optimization), inlining is disabled. In software rendering
where you might draw 1,000,000+ pixels per frame, calling a non-inlined function that creates and matches enums in an
inner loop will noticeably degrade framerates.

This is optimised away in later lessons by checking beforehand rather than when the pixel is set.

### 2. Generic / Trait-Based 2D Grid (`Buffer2D<T>`) vs. Concrete Color Buffer

In Lesson 0, the framebuffer was implemented using a concrete `Color` struct (`r, g, b, a: u8`) stored in a 1D `Vec<Color>`. While keeping it concrete simplified the initial setup, introducing a generic 2D container (`Buffer2D<T>`) offers several major advantages as the renderer grows:

- **Reusability for the Z-Buffer (Lesson 3)**: In Lesson 3, hidden-surface removal is solved with a Z-buffer (depth buffer). A depth buffer is structurally identical to a framebuffer—a 2D grid of `width × height` accessed via `(x, y)` with identical row-major indexing—but stores floating-point depth values (`f32`) instead of colors. A generic `Buffer2D<T>` can back both `Buffer2D<Color>` and `Buffer2D<f32>` without duplicating grid management code.
- **Memory Efficiency**: The concrete `Color` struct allocates 4 bytes per pixel. When rendering 24-bit RGB images, the fourth alpha byte is unused in memory and discarded during file export. For single-channel grayscale buffers (like shadow maps or ambient occlusion in later lessons), 4 bytes are allocated where 1 byte would suffice. Strongly typed pixel representations (e.g. `Rgb8([u8; 3])`, `Rgba8([u8; 4])`, `Gray8(u8)`) ensure memory matches the exact data requirements.
- **Zero-Cost Abstraction**: Rust monomorphizes generic types at compile time. `Buffer2D<f32>` compiles directly into flat float arrays without dynamic dispatch (`dyn Trait`) or runtime penalties, maintaining high performance in inner rasterization loops.

**Why it was deferred**: A generic buffer introduces trait constraints (such as `T: Copy + Default`) and separates serialization concerns (a generic grid does not inherently know how to encode itself into TGA format). Deferring this abstraction until Lesson 3 keeps Lesson 0 focused on basic TGA verification.


