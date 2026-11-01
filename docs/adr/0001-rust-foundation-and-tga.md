# Rust Foundation and TGA Image Pipeline

We decided to implement the software renderer using pure Rust standard library (`std` only), representing the canvas
with a strongly typed `Color` struct and 1D `Vec<Color>` row-major buffer with a bottom-left `(0, 0)` origin. This
preserves the educational "from scratch" philosophy of Dmitry Sokolov's original TinyRenderer course while eliminating
unsafe C unions and raw pointer arithmetic in favor of memory-safe, idiomatic Rust.
