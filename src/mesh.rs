use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use crate::math::Vec3;

/// Error returned when loading or parsing a Wavefront `.obj` mesh fails.
#[derive(Debug)]
pub enum MeshError {
    Io(io::Error),
    Parse { line: usize, message: String },
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error reading mesh: {err}"),
            Self::Parse { line, message } => {
                write!(f, "OBJ parse error at line {line}: {message}")
            }
        }
    }
}

impl Error for MeshError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Parse { .. } => None,
        }
    }
}

impl From<io::Error> for MeshError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// A 3D geometric surface defined by vertices and triangular faces.
///
/// # Internal Representation
///
/// Faces are stored as a flat, contiguous `Vec<[usize; 3]>` rather than dynamic
/// polygon vectors (`Vec<Vec<usize>>`):
/// - **Zero Per-Face Allocations**: Stores all faces in a single contiguous buffer on the heap,
///   eliminating thousands of tiny allocations during `.obj` parsing and avoiding pointer chasing.
/// - **Cache Locality & SIMD**: Fixed 3-vertex strides allow the compiler to unroll loops and
///   vectorize vertex lookups with direct stride arithmetic.
/// - **Pipeline Compatibility**: Later graphics pipeline stages (such as barycentric rasterization,
///   z-buffering, and fragment shading) mathematically operate exclusively on triangles.
///
/// When loading Wavefront `.obj` files, polygons with more than 3 vertices (such as quads or n-gons)
/// are decomposed into triangles at load time via fan triangulation (`[v0, vi, vi+1]`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Mesh {
    vertices: Vec<Vec3>,
    faces: Vec<[usize; 3]>,
}

impl Mesh {
    /// Returns the slice of 3D vertices in the mesh.
    pub fn vertices(&self) -> &[Vec3] {
        &self.vertices
    }

    /// Returns the slice of faces, where each face consists of three 0-based vertex indices.
    pub fn faces(&self) -> &[[usize; 3]] {
        &self.faces
    }

    /// Loads a `Mesh` from a Wavefront `.obj` file on disk.
    pub fn from_obj_file<P: AsRef<Path>>(path: P) -> Result<Self, MeshError> {
        let contents = fs::read_to_string(path)?;
        Self::from_obj_str(&contents)
    }

    /// Parses a `Mesh` from Wavefront `.obj` text.
    ///
    /// Supports `v x y z` vertex definitions and `f` face definitions (`v`, `v/vt`, `v//vn`,
    /// `v/vt/vn`) with both positive 1-based and negative relative vertex indices.
    /// Polygonal faces with more than 3 vertices are triangulated using fan triangulation.
    pub fn from_obj_str(input: &str) -> Result<Self, MeshError> {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for (idx, raw_line) in input.lines().enumerate() {
            let line_num = idx + 1;
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("v") => {
                    let vertex = parse_vertex(parts, line_num)?;
                    vertices.push(vertex);
                }
                Some("f") => {
                    parse_face(parts, vertices.len(), line_num, &mut faces)?;
                }
                _ => {}
            }
        }

        Ok(Self { vertices, faces })
    }
}

fn parse_vertex<'a>(
    mut parts: impl Iterator<Item = &'a str>,
    line_num: usize,
) -> Result<Vec3, MeshError> {
    let x = parse_coord(parts.next(), "x", line_num)?;
    let y = parse_coord(parts.next(), "y", line_num)?;
    let z = parse_coord(parts.next(), "z", line_num)?;
    Ok(Vec3::new(x, y, z))
}

fn parse_coord(token: Option<&str>, axis: &str, line_num: usize) -> Result<f32, MeshError> {
    let raw = token.ok_or_else(|| MeshError::Parse {
        line: line_num,
        message: format!("missing {axis} coordinate in vertex"),
    })?;
    raw.parse::<f32>().map_err(|_| MeshError::Parse {
        line: line_num,
        message: format!("invalid {axis} coordinate '{raw}'"),
    })
}

fn parse_face<'a>(
    parts: impl Iterator<Item = &'a str>,
    vertex_count: usize,
    line_num: usize,
    faces: &mut Vec<[usize; 3]>,
) -> Result<(), MeshError> {
    let mut indices = Vec::with_capacity(4);
    for token in parts {
        let vert_token = token.split('/').next().unwrap_or("");
        let raw_index = vert_token.parse::<isize>().map_err(|_| MeshError::Parse {
            line: line_num,
            message: format!("invalid vertex index '{token}'"),
        })?;

        let resolved = resolve_obj_index(raw_index, vertex_count, line_num)?;
        indices.push(resolved);
    }

    if indices.len() < 3 {
        return Err(MeshError::Parse {
            line: line_num,
            message: "face must reference at least 3 vertices".to_string(),
        });
    }

    for i in 1..(indices.len() - 1) {
        faces.push([indices[0], indices[i], indices[i + 1]]);
    }

    Ok(())
}

fn resolve_obj_index(
    raw_index: isize,
    vertex_count: usize,
    line_num: usize,
) -> Result<usize, MeshError> {
    let zero_based = if raw_index > 0 {
        (raw_index - 1) as usize
    } else if raw_index < 0 {
        let offset = vertex_count as isize + raw_index;
        if offset < 0 {
            return Err(MeshError::Parse {
                line: line_num,
                message: format!("relative vertex index {raw_index} out of bounds"),
            });
        }
        offset as usize
    } else {
        return Err(MeshError::Parse {
            line: line_num,
            message: "OBJ vertex index cannot be 0".to_string(),
        });
    };

    if zero_based >= vertex_count {
        return Err(MeshError::Parse {
            line: line_num,
            message: format!(
                "vertex index {raw_index} out of bounds (have {vertex_count} vertices)"
            ),
        });
    }

    Ok(zero_based)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_obj_formats_and_comments() {
        let obj = "\
# sample triangle and quad
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
vt 0.5 0.5
vn 0.0 0.0 1.0
f 1 2 3
f 1/1/1 2/1/1 3//1 4/1
";
        let mesh = Mesh::from_obj_str(obj).unwrap();
        assert_eq!(mesh.vertices().len(), 4);
        assert_eq!(mesh.vertices()[2], Vec3::new(1.0, 1.0, 0.0));
        assert_eq!(mesh.faces(), &[[0, 1, 2], [0, 1, 2], [0, 2, 3]]);
    }

    #[test]
    fn test_parse_negative_relative_indices() {
        let obj = "\
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 0.0 1.0 0.0
f -3 -2 -1
";
        let mesh = Mesh::from_obj_str(obj).unwrap();
        assert_eq!(mesh.faces(), &[[0, 1, 2]]);
    }

    #[test]
    fn test_rejects_out_of_bounds_and_zero_indices() {
        assert!(Mesh::from_obj_str("v 0 0 0\nf 0 1 1\n").is_err());
        assert!(Mesh::from_obj_str("v 0 0 0\nf 1 2 1\n").is_err());
        assert!(Mesh::from_obj_str("v 0 0 0\nf -2 1 1\n").is_err());
        assert!(Mesh::from_obj_str("v 0 0 0\nv 1 0 0\nf 1 2\n").is_err());
    }
}
