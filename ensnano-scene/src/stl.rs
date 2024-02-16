//! Export the 3D scene to  binary stl file format
//! The description of binary stl format:

//! UINT8[80]    – Header                 -     80 bytes
//! UINT32       – Number of triangles    -      4 bytes

//! foreach triangle                      - 50 bytes:
//!     REAL32[3] – Normal vector             - 12 bytes
//!     REAL32[3] – Vertex 1                  - 12 bytes
//!     REAL32[3] – Vertex 2                  - 12 bytes
//!     REAL32[3] – Vertex 3                  - 12 bytes
//!     UINT16    – Attribute byte count      -  2 bytes

use std::io;
use std::io::Write;

use ensnano_design::ultraviolet::{Rotor3, Vec3, Vec4};
use ensnano_design::Design;
//use ensnano_interactor::graphics::LoopoutNucl;
use crate::view::{
    ConeInstance, Ellipsoid, Instanciable, RawDnaInstance, SlicedTubeInstance, SphereInstance,
    TubeInstance, TubeLidInstance,
};

#[derive(Debug)]
pub enum StlError {
    IOError(std::io::Error),
}

trait StlProcessing {
    fn to_stl_triangles(&self) -> Vec<StlTriangle> {
        vertices_indices_to_stl_triangles(self.transformed_vertices(), self.triangle_list_indices())
    }
    fn transformed_vertices(&self) -> Vec<[f32; 3]>;
    fn triangle_list_indices(&self) -> Vec<usize>;
}

impl StlProcessing for RawDnaInstance {
    fn to_stl_triangles(&self) -> Vec<StlTriangle> {
        match self.scale.z {
            0.0 => vec![],
            _ => vertices_indices_to_stl_triangles(
                self.transformed_vertices(),
                self.triangle_list_indices(),
            ),
        }
    }

    fn transformed_vertices(&self) -> Vec<[f32; 3]> {
        let vertices = match self.mesh {
            1 => SphereInstance::vertices(),
            2 => TubeInstance::vertices(),
            4 => SlicedTubeInstance::vertices(),
            3 => TubeLidInstance::vertices(),
            6 => ConeInstance::vertices(),
            7 => Ellipsoid::vertices(),
            _ => vec![],
        };
        let model = self.model;
        let scale = self.scale;
        vertices
            .iter()
            .map(|v| Vec3::from(v.position) * scale)
            .map(|v| model.transform_point3(v))
            .map(|v| [v[0], v[1], v[2]])
            .collect()
    }

    fn triangle_list_indices(&self) -> Vec<usize> {
        match self.mesh {
            1 => SphereInstance::indices(),
            2 => triangle_indices_from_strip(TubeInstance::indices()),
            4 => triangle_indices_from_strip(SlicedTubeInstance::indices()),
            3 => TubeLidInstance::indices(),
            6 => ConeInstance::indices(),
            7 => Ellipsoid::indices(),
            _ => vec![],
        }
        .iter()
        .map(|&x| x as usize)
        .collect()
    }
}

fn triangle_indices_from_strip(indices: Vec<u16>) -> Vec<u16> {
    let mut triangle_from_strip_indices = vec![];
    let n = indices.len();
    for i in (0..n - 2) {
        triangle_from_strip_indices.push(indices[i]);
        triangle_from_strip_indices.push(indices[i + 1]);
        triangle_from_strip_indices.push(indices[i + 2]);
    }
    triangle_from_strip_indices
}

pub fn stl_bytes_export(raw_instances: Vec<RawDnaInstance>) -> Result<Vec<u8>, StlError> {
    let triangles: Vec<StlTriangle> = raw_instances
        .iter()
        .flat_map(|raw_inst| raw_inst.to_stl_triangles())
        .collect();
    let mut bytes: Vec<u8> = vec![0; 80]; // header numer of triangles
    let triangles_number: u32 = triangles.len() as u32;
    let triangle_number = triangles_number.to_le_bytes();
    bytes.extend_from_slice(&triangle_number[0..]);
    for t in triangles {
        bytes.append(&mut t.to_bytes());
    }
    Ok(bytes)
}

#[derive(Debug, Copy, Clone)]
struct StlTriangle {
    normal: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
}

impl StlTriangle {
    fn to_bytes(self) -> Vec<u8> {
        let mut result = self.normal.to_vec();
        result.extend(self.v1.to_vec());
        result.extend(self.v2.to_vec());
        result.extend(self.v3.to_vec());
        let mut result: Vec<u8> = result.iter().map(|x| x.to_le_bytes()).flatten().collect();
        result.push(0); // attribute bytes
        result.push(0);
        result
    }
}

fn vertices_indices_to_stl_triangles(
    vertices: Vec<[f32; 3]>,
    indices: Vec<usize>,
) -> Vec<StlTriangle> {
    let mut result = vec![];
    let n = indices.len();
    for i in (0..n).step_by(3) {
        result.push(StlTriangle {
            normal: [0., 0., 0.],
            v1: vertices[indices[i]].clone(),
            v2: vertices[indices[i + 1]].clone(),
            v3: vertices[indices[i + 2]].clone(),
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stl_file_from_triangles(path: &str, triangles: Vec<StlTriangle>) -> Result<(), io::Error> {
        let mut out_file = std::fs::File::create(path)?;
        let mut bytes: Vec<u8> = vec![0; 80]; // header numer of triangles
        let mut triangles_number: u32 = triangles.len() as u32;
        let triangle_number = triangles_number.to_le_bytes();
        bytes.extend_from_slice(&triangle_number[0..]);
        for t in triangles {
            bytes.append(&mut t.to_bytes());
        }
        out_file.write_all(&bytes)?;
        Ok(())
    }
    #[test]
    fn empty_stl_test() {
        assert!(stl_file_from_triangles("blop.stl", vec![]).is_ok());
    }

    #[test]
    fn triangle_stl_test() {
        let t = StlTriangle {
            normal: [0., 0., 0.],
            v1: [0., 0., 0.],
            v2: [0., 1., 0.],
            v3: [1., 0., 0.],
        };
        let t2 = StlTriangle {
            normal: [0., 0., 0.],
            v1: [1., 0., 0.],
            v2: [0., 1., 0.],
            v3: [0., 0., 2.],
        };
        assert!(stl_file_from_triangles("blop_triangle.stl", vec![t, t2]).is_ok());
    }

    #[test]
    fn vi_stl() {
        assert_eq!(
            format!(
                "{:?}",
                vertices_indices_to_stl_triangles(
                    vec![[0., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
                    vec![0, 1, 2, 1, 2, 0]
                )[1]
                .v1
            ),
            "[0.0, 1.0, 0.0]"
        );
    }

    // fn stl_raw() {
    //     let rawi = RawDnaInstance {
    //         model: Mat4::identity(),
    //         scale: Vec3::from([1.0, 1.0, 2.3]),
    //         color: Vec4::zero(),
    //         id: 1,
    //         inversed_model: Mat4::identity(),
    //         prev: Vec3::zero(),
    //         mesh: 1,
    //         next: Vec3::zero(),
    //     };
    //     assert!(stl_file_from_triangles("raw.stl", rawi.to_stl_triangles()))
    // }

    // #[test]
    // fn lots_of_centers_to_stl() {
    //     let ts = (0..500).map(|i| Vec3::from([i as f32, i as f32, i as f32]));
    //     let ts = ts.map(|c| stl_obj_to_triangles(c, 1.0)).flatten().collect();
    //     assert!(stl_bytes_from_triangles("many_nucl.stl", ts).is_ok())
    // }
}
