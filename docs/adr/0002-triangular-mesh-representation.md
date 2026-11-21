# Triangular Mesh Representation and Load-Time Triangulation

We decided to represent mesh faces as a flat contiguous vector of fixed 3-element arrays (`Vec<[usize; 3]>`), triangulating polygonal faces at load time using fan triangulation. This eliminates thousands of per-face heap allocations incurred by dynamic polygon representations (`Vec<Vec<usize>>`) while directly aligning with downstream rasterization stages (barycentric interpolation, z-buffering, fragment shading) which fundamentally require triangular primitives.

## Considered Options

- **`Vec<Vec<usize>>` (Dynamic Polygons)**: Preserves original OBJ face arity (triangles, quads, n-gons) directly, but incurs a separate heap allocation for every individual face (e.g. 2,492 heap allocations for a simple head model) and degrades cache locality with pointer indirection during rendering loops.
- **Flat Index Buffer + Offsets (Contiguous Ragged Array)**: Stores all indices in one `Vec<usize>` with a secondary `Vec<usize>` of face offsets. Eliminates allocations and preserves n-gon topology, but adds indirection lookups per face, loses compile-time SIMD/unrolling guarantees, and forces the rasterizer to dynamically triangulate n-gons on every frame anyway once solid shading and z-buffering are introduced.
- **Fixed-Size Triangles (`Vec<[usize; 3]>`)**: Allocates a single contiguous buffer for all faces. Faces have compile-time fixed strides enabling direct offset indexing and compiler unrolling. General polygons are triangulated once during `.obj` parsing via fan triangulation.
