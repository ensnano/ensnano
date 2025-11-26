use super::*;
use rand::Rng as _;
use ultraviolet::Vec2;

const DEFAULT_BEZIER_TANGENT_NORM: f32 = 1. / 3.;

pub(crate) trait BezierEndCoordinateUnit:
    std::ops::Add<Self, Output = Self>
    + std::ops::Sub<Self, Output = Self>
    + std::ops::Mul<f32, Output = Self>
    + std::ops::Div<f32, Output = Self>
    + Sized
    + Clone
    + Copy
{
    fn instantiate_bezier_end(
        position: Self,
        vector_in: Self,
        vector_out: Self,
    ) -> BezierEndCoordinates;
    fn from_projection(v: Vec3) -> Self;
    fn one() -> Self;
}

impl BezierEndCoordinateUnit for Vec3 {
    fn instantiate_bezier_end(
        position: Self,
        vector_in: Self,
        vector_out: Self,
    ) -> BezierEndCoordinates {
        BezierEndCoordinates {
            position,
            vector_in,
            vector_out,
        }
    }

    fn from_projection(v: Vec3) -> Self {
        v
    }

    fn one() -> Self {
        Self::one()
    }
}

impl BezierEndCoordinateUnit for Vec2 {
    fn instantiate_bezier_end(
        position: Self,
        vector_in: Self,
        vector_out: Self,
    ) -> BezierEndCoordinates {
        let to_vec3 = |v: Self| Vec3 {
            x: v.x,
            y: v.y,
            z: 0.0,
        };
        BezierEndCoordinates {
            position: to_vec3(position),
            vector_in: to_vec3(vector_in),
            vector_out: to_vec3(vector_out),
        }
    }

    fn one() -> Self {
        Self::one()
    }

    fn from_projection(v: Vec3) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// An object capable of instantiating PieceWiseBezier curves.
pub(crate) trait PieceWiseBezierInstantiator<T: BezierEndCoordinateUnit> {
    fn nb_vertices(&self) -> usize;
    fn position(&self, i: usize) -> Option<T>;
    fn vector_in(&self, i: usize) -> Option<T>;
    fn vector_out(&self, i: usize) -> Option<T>;
    fn is_cyclic(&self) -> bool;

    fn instantiate(&self) -> Option<InstantiatedPiecewiseBezier> {
        let descriptor = if self.nb_vertices() > 2 {
            let n = self.nb_vertices();
            let idx_iterator: Box<dyn Iterator<Item = ((usize, usize), usize)>> =
                if self.is_cyclic() {
                    Box::new(
                        (0..n)
                            .cycle()
                            .skip(n - 1)
                            .zip((0..n).cycle().take(n + 1))
                            .zip((0..n).cycle().skip(1)),
                    )
                } else {
                    // iterate from 0 to n-1 and add manually the first and last vertices
                    // afterwards
                    Box::new((0..n).zip((0..n).skip(1)).zip((0..n).skip(2)))
                };
            let mut bezier_points: Vec<_> = idx_iterator
                .filter_map(|((idx_from, idx), idx_to)| {
                    let pos_from = self.position(idx_from)?;
                    let pos = self.position(idx)?;
                    let pos_to = self.position(idx_to)?;
                    let vector_in = self
                        .vector_in(idx)
                        .unwrap_or((pos_to - pos_from) * DEFAULT_BEZIER_TANGENT_NORM);
                    let vector_out = self
                        .vector_out(idx)
                        .unwrap_or((pos_to - pos_from) * DEFAULT_BEZIER_TANGENT_NORM);

                    Some(T::instantiate_bezier_end(pos, vector_in, vector_out))
                })
                .collect();
            if !self.is_cyclic() {
                // Add manually the first and last vertices
                let first_point = {
                    let second_point = bezier_points.first()?;
                    let pos = self.position(0)?;
                    let control =
                        T::from_projection(second_point.position - second_point.vector_in);

                    let vector_out = self.vector_out(0).unwrap_or((control - pos) / 2.);

                    let vector_in = self.vector_in(0).unwrap_or((control - pos) / 2.);

                    T::instantiate_bezier_end(pos, vector_in, vector_out)
                };
                bezier_points.insert(0, first_point);
                let last_point = {
                    let second_to_last_point = bezier_points.last()?;
                    // Ok to unwrap because vertices has length > 2
                    let pos = self.position(self.nb_vertices() - 1)?;
                    let control = T::from_projection(
                        second_to_last_point.position + second_to_last_point.vector_out,
                    );
                    let vector_out = self
                        .vector_out(self.nb_vertices() - 1)
                        .unwrap_or((pos - control) / 2.);

                    let vector_in = self
                        .vector_in(self.nb_vertices() - 1)
                        .unwrap_or((pos - control) / 2.);
                    T::instantiate_bezier_end(pos, vector_in, vector_out)
                };
                bezier_points.push(last_point);
            } else {
                bezier_points.pop();
            }
            Some(bezier_points)
        } else if self.nb_vertices() == 2 {
            let pos_first = self.position(0)?;
            let pos_last = self.position(1)?;
            let default_vec = (pos_last - pos_first) / 3.;
            let vec_in_first = self.vector_in(0).unwrap_or(default_vec);
            let vec_out_first = self.vector_out(0).unwrap_or(default_vec);
            let vec_in_last = self.vector_in(1).unwrap_or(default_vec);
            let vec_out_last = self.vector_out(1).unwrap_or(default_vec);
            Some(vec![
                T::instantiate_bezier_end(pos_first, vec_in_first, vec_out_first),
                T::instantiate_bezier_end(pos_last, vec_in_last, vec_out_last),
            ])
        } else if self.nb_vertices() == 1 {
            let pos_first = self.position(0)?;
            Some(vec![T::instantiate_bezier_end(
                pos_first,
                T::one() * f32::NAN,
                T::one() * f32::NAN,
            )])
        } else {
            None
        }?;
        let mut rng = rand::thread_rng();
        let t_max = if self.is_cyclic() {
            Some(descriptor.len() as f64)
        } else {
            Some(descriptor.len() as f64 - 1.)
        };
        Some(InstantiatedPiecewiseBezier {
            t_min: None,
            t_max,
            ends: descriptor,
            is_cyclic: self.is_cyclic(),
            id: rng.r#gen(),
            discretize_quickly: false,
        })
    }
}
