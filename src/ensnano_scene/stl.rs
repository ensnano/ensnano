//! Export the 3D scene to  binary stl file format
//! The description of binary stl format:

//! [u8; 80]     – Header                 - 80 bytes
//! u32          – Number of triangles    -  4 bytes
//! foreach triangle                      - 50 bytes:
//!     [f32; 3] – Normal vector          - 12 bytes
//!     [f32; 3] – Vertex 1               - 12 bytes
//!     [f32; 3] – Vertex 2               - 12 bytes
//!     [f32; 3] – Vertex 3               - 12 bytes
//!     u16      – Attribute byte count   -  2 bytes

use super::view::{
    ConeInstance, Ellipsoid, Instantiable, Mesh, Mesh::*, RawDnaInstance, SlicedTubeInstance,
    SphereInstance, TubeInstance, TubeLidInstance,
};
use crate::ensnano_consts::NB_RAY_TUBE;
use ultraviolet::{Mat3, Vec3};

impl RawDnaInstance {
    fn to_stl_triangles(self) -> Vec<StlTriangle> {
        if self.scale.z.abs() < 1e-6 {
            vec![]
        } else {
            vertices_indices_to_stl_triangles(
                self.transformed_vertices_normal(),
                self.triangle_list_indices(),
            )
        }
    }

    fn transformed_vertices_normal(&self) -> Vec<([f32; 3], [f32; 3])> {
        let mesh = Mesh::try_from(self.mesh).unwrap();
        let vertices_normal = match mesh {
            Sphere => SphereInstance::vertices(),
            Tube => TubeInstance::vertices(),
            SlicedTube => SlicedTubeInstance::vertices(),
            TubeLid => TubeLidInstance::vertices(),
            Prime3Cone => ConeInstance::vertices(),
            BaseEllipsoid => Ellipsoid::vertices(),
            _ => vec![],
        };
        let model = self.model;
        let scale = self.scale;
        let m4 = self.inversed_model.transposed();
        let normal_matrix = Mat3::from([
            m4[0][0], m4[0][1], m4[0][2], m4[1][0], m4[1][1], m4[1][2], m4[2][0], m4[2][1],
            m4[2][2],
        ]);
        if mesh != SlicedTube {
            vertices_normal
                .iter()
                .map(|v| (Vec3::from(v.position) * scale, Vec3::from(v.normal)))
                .map(|(v, n)| (model.transform_point3(v), (normal_matrix * n).normalized()))
                .map(|(v, n)| ([v[0], v[1], v[2]], [n[0], n[1], n[2]]))
                .collect()
        } else {
            // CODE TRANSLATED IN RUST FROM SHADER sliced_tube.vert
            let mut ret = Vec::new();
            // Left side
            if self.prev.mag() > 1e-5 {
                // left must be adjusted
                let prev = self.prev.normalized();
                // compute the normal to the intersection plane
                let vec_x = Vec3::unit_x();
                let bisector = (prev - vec_x).normalized();
                let perp_vec = prev.cross(vec_x);
                let plane_normal = bisector.cross(perp_vec).normalized();
                // project the point on the intersection plane
                for i in 0..NB_RAY_TUBE {
                    let (mut position, normal) = (
                        Vec3::from(vertices_normal[i].position) * scale,
                        Vec3::from(vertices_normal[i].normal),
                    );
                    position.x -= (plane_normal.y * position.y + plane_normal.z * position.z)
                        / plane_normal.x;
                    // compute the normal by projecting the tangent on the intersection plane and taking the cross product to get a normal in the plane and perpendicular to the tangent
                    let mut tangent = Vec3::new(0., normal[2], -normal[1]);
                    tangent.x =
                        -(plane_normal.y * tangent.y + plane_normal.z * tangent.z) / plane_normal.x;
                    let normal = tangent.cross(plane_normal).normalized();
                    let p = model.transform_point3(position);
                    let n = normal_matrix * normal;
                    ret.push(([p.x, p.y, p.z], [n.x, n.y, n.z]));
                }
            } else {
                for i in 0..NB_RAY_TUBE {
                    let v_n = vertices_normal[i];
                    let p = model.transform_point3(Vec3::from(v_n.position) * scale);
                    let n = normal_matrix * Vec3::from(v_n.normal);
                    ret.push(([p.x, p.y, p.z], [n.x, n.y, n.z]));
                }
            }
            // Middle
            for i in NB_RAY_TUBE..2 * NB_RAY_TUBE {
                let v_n = vertices_normal[i];
                let p = model.transform_point3(Vec3::from(v_n.position) * scale);
                let n = normal_matrix * Vec3::from(v_n.normal);
                ret.push(([p.x, p.y, p.z], [n.x, n.y, n.z]));
            }
            // right side
            if self.next.mag() > 1e-5 {
                // left must be adjusted
                let next = self.next.normalized();
                // compute the normal to the intersection plane
                let vec_x = Vec3::unit_x();
                let bisector = (vec_x - next).normalized();
                let perp_vec = vec_x.cross(next);
                let plane_normal = bisector.cross(perp_vec).normalized();
                // project the point on the intersection plane
                for i in 2 * NB_RAY_TUBE..3 * NB_RAY_TUBE {
                    let (mut position, normal) = (
                        Vec3::from(vertices_normal[i].position) * scale,
                        Vec3::from(vertices_normal[i].normal),
                    );
                    position.x -= (plane_normal.y * position.y + plane_normal.z * position.z)
                        / plane_normal.x;
                    // compute the normal by projecting the tangent on the intersection plane and taking the cross product to get a normal in the plane and perpendicular to the tangent
                    let mut tangent = Vec3::new(0., -normal.z, normal.y);
                    tangent.x =
                        -(plane_normal.y * tangent.y + plane_normal.z * tangent.z) / plane_normal.x;
                    let normal = plane_normal.cross(tangent).normalized();
                    let p = model.transform_point3(position);
                    let n = normal_matrix * normal;
                    ret.push(([p.x, p.y, p.z], [n.x, n.y, n.z]));
                }
            } else {
                for i in 2 * NB_RAY_TUBE..3 * NB_RAY_TUBE {
                    let v_n = vertices_normal[i];
                    let p = model.transform_point3(Vec3::from(v_n.position) * scale);
                    let n = normal_matrix * Vec3::from(v_n.normal);
                    ret.push(([p.x, p.y, p.z], [n.x, n.y, n.z]));
                }
            }
            ret
        }
    }

    fn triangle_list_indices(&self) -> Vec<usize> {
        let mesh = Mesh::try_from(self.mesh).unwrap();
        match mesh {
            Sphere => SphereInstance::indices(),
            Tube => triangle_indices_from_strip(TubeInstance::indices()),
            SlicedTube => triangle_indices_from_strip(SlicedTubeInstance::indices()),
            TubeLid => TubeLidInstance::indices(),
            Prime3Cone => ConeInstance::indices(),
            BaseEllipsoid => Ellipsoid::indices(),
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
    for i in 0..n - 2 {
        if i % 2 == 0 {
            triangle_from_strip_indices.push(indices[i]);
            triangle_from_strip_indices.push(indices[i + 1]);
            triangle_from_strip_indices.push(indices[i + 2]);
        } else {
            triangle_from_strip_indices.push(indices[i + 1]);
            triangle_from_strip_indices.push(indices[i]);
            triangle_from_strip_indices.push(indices[i + 2]);
        };
    }
    triangle_from_strip_indices
}

pub fn stl_bytes_export(raw_instances: Vec<RawDnaInstance>) -> Vec<u8> {
    let triangles: Vec<StlTriangle> = raw_instances
        .iter()
        .flat_map(|raw_inst| raw_inst.to_stl_triangles())
        .collect();
    let mut bytes: Vec<u8> = vec![0; 80]; // header number of triangles
    let triangles_number: u32 = triangles.len() as u32;
    let triangle_number = triangles_number.to_le_bytes();
    bytes.extend_from_slice(&triangle_number[0..]);
    for t in triangles {
        bytes.append(&mut t.to_bytes());
    }
    bytes
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
        let mut result: Vec<u8> = result.iter().flat_map(|x| x.to_le_bytes()).collect();
        result.push(0); // attribute bytes
        result.push(0);
        result
    }
}

fn vertices_indices_to_stl_triangles(
    vertices_normal: Vec<([f32; 3], [f32; 3])>,
    indices: Vec<usize>,
) -> Vec<StlTriangle> {
    let mut result = vec![];
    let n = indices.len();
    for i in (0..n).step_by(3) {
        let (v1, v2, v3) = (
            vertices_normal[indices[i]].0,
            vertices_normal[indices[i + 1]].0,
            vertices_normal[indices[i + 2]].0,
        );
        let normal = vertices_normal[indices[i + 1]].1;
        // ( // average of normals gives worse results:
        // Vec3::from(vertices_normal[indices[i]].1)
        // + Vec3::from(vertices_normal[indices[i + 1]].1)
        // + Vec3::from(vertices_normal[indices[i + 2]].1)
        // ).normalized();
        result.push(StlTriangle { normal, v1, v2, v3 });
    }
    result
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::io::{self, Write as _},
    };

    fn stl_file_from_triangles(path: &str, triangles: Vec<StlTriangle>) -> Result<(), io::Error> {
        let mut out_file = std::fs::File::create(path)?;
        let mut bytes: Vec<u8> = vec![0; 80]; // header number of triangles
        let triangles_number: u32 = triangles.len() as u32;
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
        assert!(stl_file_from_triangles("blop.stl", vec![]).is_ok()); // cspell: disable-line
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
        assert!(stl_file_from_triangles("blop_triangle.stl", vec![t, t2]).is_ok()); // cspell: disable-line
    }
}
