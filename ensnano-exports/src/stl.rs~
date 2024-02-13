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

use ensnano_design::ultraviolet::{Vec3, Vec4};
use ensnano_design::Design;
//use ensnano_interactor::graphics::LoopoutNucl;
use ensnano_scene::view::{Instanciable, SphereInstance};
use ensnano_scene::AppState;
use ensnano_scene::Scene;

/*pub trait StlConversion<T: Instanciable> {
    fn get_stl_triangles(self) -> Vec<StlTriangle> {
        vec![]
    }
}*/

#[derive(Debug)]
pub enum StlError {
    IOError(std::io::Error),
}

pub fn stl_bytes_export(centers: Vec<Vec3>) -> Result<Vec<u8>, StlError> {
    let triangles: Vec<StlTriangle> = centers
        .iter()
        .map(|c| sphere_center_to_stl_triangles(*c))
        .flatten()
        .collect();
    let mut bytes: Vec<u8> = vec![0; 80]; // header numer of triangles
    let mut triangles_number: u32 = triangles.len() as u32;
    let triangle_number = triangles_number.to_le_bytes();
    bytes.extend_from_slice(&triangle_number[0..]);
    for t in triangles {
        bytes.append(&mut triangle_to_bytes(t));
    }
    Ok(bytes)
}

pub fn save_stl_from_scene<S: AppState>(path: &str, scene: Scene<S>) -> Result<(), io::Error> {
    let centers = scene.get_nucl_centers_for_stl();
    let all_triangles = centers
        .iter()
        .map(|c| sphere_center_to_stl_triangles(*c))
        .flatten()
        .collect();
    stl_bytes_from_triangles(path, all_triangles)
}

fn sphere_center_to_stl_triangles(c: Vec3) -> Vec<StlTriangle> {
    let translated_vertices: Vec<[f32; 3]> = SphereInstance::vertices()
        .clone()
        .iter()
        .map(|v| {
            [
                v.position[0] + c[0],
                v.position[1] + c[1],
                v.position[2] + c[2],
            ]
        })
        .collect();
    vertices_indices_to_stl_triangles(
        translated_vertices,
        SphereInstance::indices()
            .iter()
            .map(|x| usize::from(*x))
            .collect(),
    )
}

// fn sphere_instance_to_stl_triangles(si: SphereInstance) -> Vec<StlTriangle> {
//     let t = si.position;
//     let translated_vertices: Vec<[f32; 3]> = SphereInstance::vertices()
//         .clone()
//         .iter()
//         .map(|v| {
//             [
//                 v.position[0] + t[0],
//                 v.position[1] + t[1],
//                 v.position[2] + t[2],
//             ]
//         })
//         .colect();
//     vertices_indices_to_stl_triangles(
//         translated_vertices,
//         SphereInstance::indices()
//             .iter()
//             .map(|x| usize::from(*x))
//             .collect(),
//     )
// }

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

#[derive(Debug, Copy, Clone)]
struct StlTriangle {
    normal: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
}

fn triangle_to_bytes(t: StlTriangle) -> Vec<u8> {
    let mut result = t.normal.to_vec();
    result.extend(t.v1.to_vec());
    result.extend(t.v2.to_vec());
    result.extend(t.v3.to_vec());
    let mut result: Vec<u8> = result.iter().map(|x| x.to_le_bytes()).flatten().collect();
    result.push(0); // attribute bytes
    result.push(0);
    result
}

fn stl_bytes_from_triangles(path: &str, triangles: Vec<StlTriangle>) -> Result<(), io::Error> {
    let mut out_file = std::fs::File::create(path)?;
    let mut bytes: Vec<u8> = vec![0; 80]; // header numer of triangles
    let mut triangles_number: u32 = triangles.len() as u32;
    let triangle_number = triangles_number.to_le_bytes();
    bytes.extend_from_slice(&triangle_number[0..]);
    for t in triangles {
        bytes.append(&mut triangle_to_bytes(t));
    }
    out_file.write_all(&bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_stl_test() {
        assert!(stl_bytes_from_triangles("blop.stl", vec![]).is_ok());
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
        assert!(stl_bytes_from_triangles("blop_triangle.stl", vec![t, t2]).is_ok());
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

    #[test]
    fn center_to_stl() {
        let mut t = sphere_center_to_stl_triangles(Vec3::from([0., 0., 0.]));
        let t2 = sphere_center_to_stl_triangles(Vec3::from([4., 0., 0.]));
        t.extend(t2);
        assert!(stl_bytes_from_triangles("nucl.stl", t).is_ok())
    }

    #[test]
    fn lots_of_centers_to_stl() {
        let ts = (0..500).map(|i| Vec3::from([i as f32, i as f32, i as f32]));
        let ts = ts
            .map(|c| sphere_center_to_stl_triangles(c))
            .flatten()
            .collect();
        assert!(stl_bytes_from_triangles("many_nucl.stl", ts).is_ok())
    }
}
