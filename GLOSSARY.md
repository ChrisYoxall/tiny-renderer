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

**Rasterization**:
The process of converting geometric primitives (such as line segments or triangles) into discrete pixels in a framebuffer.
_Avoid_: Drawing, painting, rendering (when referring specifically to pixel conversion)

**Line Segment**:
A finite straight primitive defined by two endpoints in screen coordinates.
_Avoid_: Line, edge (unless referring specifically to a mesh polygon boundary)

**Mesh**:
A 3D geometric surface defined by a collection of vertices and faces.
_Avoid_: Model, object

**Vertex**:
A 3D position in normalized model coordinates `[-1, 1]³`.
_Avoid_: Point, corner

**Face**:
An ordered list of vertex indices forming a closed surface element.
_Avoid_: Polygon, triangle (when referring to general `.obj` faces)

**Wireframe**:
A rendering mode that rasterizes only the edges of each face as line segments.
_Avoid_: Skeleton, outline
