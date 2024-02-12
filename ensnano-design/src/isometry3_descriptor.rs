/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use mathru::algebra::abstr::Identity;
use std::f32::consts::PI;
use ultraviolet::*;

use std::str::FromStr;

use std::collections::HashMap;

use crate::{ParsePointError};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Isometry3DescriptorItem {
    Identity,                             // %id
    TranslateBy(Vec3),                    // %tr(xx,yy,zz)
    TranslateX(f32),                      // %tX(xx)
    TranslateY(f32),                      // %tY(yy)
    TranslateZ(f32),                      // %tZ(zz)
    RotateFromToAround(Vec3, Vec3, Vec3), // %rot(x1,y1,z1,x2,y2,z2,xc,yc,zc)
    RotateFromTo(Vec3, Vec3),             // around Vec3::zero // // %rot(x1,y1,z1,x2,y2,z2)
    RotateYZByAround(f32, Vec3),          // %rotYZ(a,xc,yc,zc)
    RotateZXByAround(f32, Vec3),          // %rotZX(a,xc,yc,zc)
    RotateXYByAround(f32, Vec3),          // %rotXY(a,xc,yc,zc)
    RotateYZBy(f32),                      // around Vec3::zero // %rotYZ(a)
    RotateZXBy(f32),                      // around Vec3::zero // %rotZX(a)
    RotateXYBy(f32),                      // around Vec3::zero // %rotXY(a)
}

pub trait Parsef32s {
    fn parse_f32s_separated_by_commas_parenthesis_or_space(s: &str) -> Vec<f32>;
    fn parse_f32s_separated_by_commas_parenthesis_or_space_with_variables(s: &str, variables: Option<&HashMap<String, f32>>) -> Vec<f32>;
}

impl Parsef32s for Isometry3DescriptorItem {
    fn parse_f32s_separated_by_commas_parenthesis_or_space(s: &str) -> Vec<f32> {
        let s = s.split(&[',', '(', ')', ' ']);
        let mut ret = Vec::new();
        for t in s {
            let t = t.trim();
            if let Ok(f) = f32::from_str(t) {
                ret.push(f);
            }
        }
        return ret;
    }

    fn parse_f32s_separated_by_commas_parenthesis_or_space_with_variables(s: &str, variables: Option<&HashMap<String, f32>>) -> Vec<f32> {
        let s = s.split(&[',', '(', ')', ' ']);
        let mut ret = Vec::new();
        for t in s {
            let t = t.trim();
            if let Ok(f) = f32::from_str(t) {
                ret.push(f);
            } else if let Some(variables) = variables {
                let parsed_t = t.split(&[' ','*']).filter(|s| *s != "").collect::<Vec<&str>>();
                if parsed_t.len() == 2 {
                    if let Ok(value) = f32::from_str(parsed_t[0]) {
                        if let Some(v) = variables.get(parsed_t[1]) {
                            ret.push(value * v);
                        }
                    }
                }
            }
        }
        return ret;
    }
}


impl Isometry3DescriptorItem {

    /// Parse an Isometry3DescriptorItem:
    /// - %id for Identity
    /// - %tr(xx,yy,zz) for Translation(Vec3::new(xx,yy,zz)) where xx,yy,zz can be f32'*'variable_name where variable_name is in variables
    /// - %tX(xx) for TranslationX(xx)
    /// - %tY(yy) for TranslationY(yy)
    /// - %tZ(zz) for TranslationZ(zz)
    /// - %rot(x1,y1,z1,x2,y2,z2,xc,yc,zc) for RotateFromToAround(Vec3::new(x1,y1,z1),Vec3::new(x2,y2,z2),Vec3::new(xc,yc,zc))
    /// - %rot(x1,y1,z1,x2,y2,z2) for RotateFromTo(Vec3::new(x1,y1,z1),Vec3::new(x2,y2,z2))
    /// - %rotXY(a,xc,yc,zc) for RotateXYByAround(a,Vec3::new(xc,yc,zc))
    /// - %rotXY(a) for RotateXYBy(a)
    /// - %rotYZ(a,xc,yc,zc) for RotateYZByAround(a,Vec3::new(xc,yc,zc))
    /// - %rotYZ(a) for RotateYZBy(a)
    /// - %rotZX(a,xc,yc,zc) for RotateZXByAround(a,Vec3::new(xc,yc,zc))
    /// - %rotZX(a) for RotateZXBy(a)
    fn from_str_with_variables(s: &str, variables: Option<&HashMap<String, f32>>) -> Result<Self, ParsePointError> {
        let s = s.trim();

        if s.starts_with("%id") {
            return Ok(Self::Identity);
        } else if s.starts_with("%tr(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space_with_variables(&s[4..], variables);
            if args.len() == 3 {
                return Ok(Self::TranslateBy(Vec3::new(args[0], args[1], args[2])));
            }
        } else if s.starts_with("%tX(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space_with_variables(&s[4..], variables);
            if args.len() == 1 {
                return Ok(Self::TranslateX(args[0]));
            }
        } else if s.starts_with("%tY(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space_with_variables(&s[4..], variables);
            if args.len() == 1 {
                return Ok(Self::TranslateY(args[0]));
            }
        } else if s.starts_with("%tZ(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space_with_variables(&s[4..], variables);
            if args.len() == 1 {
                return Ok(Self::TranslateZ(args[0]));
            }
        } else if s.starts_with("%rot(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space(&s[5..]);
            match args.len() {
                9 => {
                    return Ok(Self::RotateFromToAround(
                        Vec3::new(args[0], args[1], args[2]),
                        Vec3::new(args[3], args[4], args[5]),
                        Vec3::new(args[6], args[7], args[8]),
                    ))
                }
                6 => {
                    return Ok(Self::RotateFromTo(
                        Vec3::new(args[0], args[1], args[2]),
                        Vec3::new(args[3], args[4], args[5]),
                    ))
                }
                _ => (),
            }
        } else if s.starts_with("%rotYZ(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space(&s[7..]);
            match args.len() {
                1 => return Ok(Self::RotateYZBy(args[0])),
                4 => {
                    return Ok(Self::RotateYZByAround(
                        args[0],
                        Vec3::new(args[1], args[2], args[3]),
                    ))
                }
                _ => (),
            }
        } else if s.starts_with("%rotZX(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space(&s[7..]);
            match args.len() {
                1 => return Ok(Self::RotateZXBy(args[0])),
                4 => {
                    return Ok(Self::RotateZXByAround(
                        args[0],
                        Vec3::new(args[1], args[2], args[3]),
                    ))
                }
                _ => (),
            }
        } else if s.starts_with("%rotXY(") {
            let args = Self::parse_f32s_separated_by_commas_parenthesis_or_space(&s[7..]);
            match args.len() {
                1 => return Ok(Self::RotateXYBy(args[0])),
                4 => {
                    return Ok(Self::RotateXYByAround(
                        args[0],
                        Vec3::new(args[1], args[2], args[3]),
                    ))
                }
                _ => (),
            }
        }
        return Err(ParsePointError);
    }
}

impl FromStr for Isometry3DescriptorItem {
    type Err = ParsePointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Self::from_str_with_variables(s, None);
    }
}

pub type Isometry3Descriptor = Vec<Isometry3DescriptorItem>;

pub trait Isometry3MissingMethods {
    fn from_descriptor(descriptor: &Isometry3Descriptor) -> Isometry3;
    fn translation(u: Vec3) -> Isometry3;
    fn rotation_from_to_around(u: Vec3, v: Vec3, c: Vec3) -> Isometry3;
    fn rotation_yz_by_around(degree: f32, c: Vec3) -> Isometry3;
    fn rotation_zx_by_around(degree: f32, c: Vec3) -> Isometry3;
    fn rotation_xy_by_around(degree: f32, c: Vec3) -> Isometry3;
    fn composed_with(&self, iso2: Isometry3) -> Isometry3;
    fn from_str(s: &str) -> Isometry3;
    fn from_str_with_variables(s: &str, variables: Option<&HashMap<String, f32>>) -> Isometry3;
}

impl Isometry3MissingMethods for Isometry3 {
    fn from_descriptor(descriptor: &Isometry3Descriptor) -> Isometry3 {
        let mut isometry3 = Isometry3::identity();

        for item in descriptor.into_iter() {
            match item {
                Isometry3DescriptorItem::Identity => (),
                Isometry3DescriptorItem::TranslateBy(u) => {
                    isometry3 = isometry3.composed_with(Isometry3::translation(*u))
                }
                Isometry3DescriptorItem::TranslateX(dx) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::translation(Vec3::new(*dx, 0., 0.)))
                }
                Isometry3DescriptorItem::TranslateY(dy) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::translation(Vec3::new(0., *dy, 0.)))
                }
                Isometry3DescriptorItem::TranslateZ(dz) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::translation(Vec3::new(0., 0., *dz)))
                }
                Isometry3DescriptorItem::RotateFromToAround(u, v, c) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::rotation_from_to_around(*u, *v, *c))
                }
                Isometry3DescriptorItem::RotateFromTo(u, v) => {
                    isometry3 = isometry3.composed_with(Isometry3::rotation_from_to_around(
                        *u,
                        *v,
                        Vec3::zero(),
                    ))
                }
                Isometry3DescriptorItem::RotateYZByAround(degree, c) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::rotation_yz_by_around(*degree, *c))
                }
                Isometry3DescriptorItem::RotateZXByAround(degree, c) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::rotation_zx_by_around(*degree, *c))
                }
                Isometry3DescriptorItem::RotateXYByAround(degree, c) => {
                    isometry3 =
                        isometry3.composed_with(Isometry3::rotation_xy_by_around(*degree, *c))
                }
                Isometry3DescriptorItem::RotateYZBy(degree) => {
                    isometry3 = isometry3
                        .composed_with(Isometry3::rotation_yz_by_around(*degree, Vec3::zero()))
                }
                Isometry3DescriptorItem::RotateZXBy(degree) => {
                    isometry3 = isometry3
                        .composed_with(Isometry3::rotation_zx_by_around(*degree, Vec3::zero()))
                }
                Isometry3DescriptorItem::RotateXYBy(degree) => {
                    isometry3 = isometry3
                        .composed_with(Isometry3::rotation_xy_by_around(*degree, Vec3::zero()))
                }
            }
        }

        return isometry3;
    }

    fn translation(u: Vec3) -> Isometry3 {
        Isometry3 {
            translation: u,
            rotation: Rotor3::identity(),
        }
    }

    fn rotation_from_to_around(u: Vec3, v: Vec3, c: Vec3) -> Isometry3 {
        let rotor = Rotor3::from_rotation_between(u, v);
        Isometry3 {
            translation: c - c.rotated_by(rotor),
            rotation: rotor,
        }
    }

    fn rotation_yz_by_around(degree: f32, c: Vec3) -> Isometry3 {
        let α = PI * degree / 180.;
        let rotor =
            Rotor3::from_rotation_between(Vec3::new(0., 1., 0.), Vec3::new(0., α.cos(), α.sin()));
        Isometry3 {
            translation: c - c.rotated_by(rotor),
            rotation: rotor,
        }
    }

    fn rotation_zx_by_around(degree: f32, c: Vec3) -> Isometry3 {
        let α = PI * degree / 180.;
        let rotor =
            Rotor3::from_rotation_between(Vec3::new(0., 0., 1.), Vec3::new(α.sin(), 0., α.cos()));
        Isometry3 {
            translation: c - c.rotated_by(rotor),
            rotation: rotor,
        }
    }

    fn rotation_xy_by_around(degree: f32, c: Vec3) -> Isometry3 {
        let α = PI * degree / 180.;
        let rotor =
            Rotor3::from_rotation_between(Vec3::new(1., 0., 0.), Vec3::new(α.cos(), α.sin(), 0.));
        Isometry3 {
            translation: c - c.rotated_by(rotor),
            rotation: rotor,
        }
    }

    fn composed_with(&self, iso2: Isometry3) -> Isometry3 {
        Isometry3 {
            translation: iso2.translation + self.translation.rotated_by(iso2.rotation),
            rotation: self.rotation * iso2.rotation,
        }
    }

    fn from_str(s: &str) -> Isometry3 {
        let descr = s
            .split(&[' ', ')'])
            .map(|x| Isometry3DescriptorItem::from_str(x))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<Isometry3DescriptorItem>>();

        println!("{:?}", descr);

        return Isometry3::from_descriptor(&descr);
    }

    fn from_str_with_variables(s: &str, variables: Option<&HashMap<String, f32>>) -> Isometry3 {
        let descr = s
            .split(&[' ', ')'])
            .map(|x| Isometry3DescriptorItem::from_str_with_variables(x, variables))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<Isometry3DescriptorItem>>();

        println!("{:?}", descr);

        return Isometry3::from_descriptor(&descr);
    }
}
