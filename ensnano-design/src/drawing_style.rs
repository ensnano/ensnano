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

use crate::MaterialColor;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DrawingAttribute {
    SphereRadius(f32),
    BondRadius(f32),
    SphereColor(u32), // with alpha
    BondColor(u32),   // with alpha
    DoubleHelixAsCylinderRadius(f32),
    DoubleHelixAsCylinderColor(u32), // with alpha
    RainbowStrand(bool),
    XoverColoring(bool),
    ColorShade(u32, Option<f64>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParsePointError;

impl FromStr for DrawingAttribute {
    type Err = ParsePointError;

    /// Parse a DrawingAttribute:
    /// - %rs / %nors for RainbowStrand(true / false) - default = false
    /// - %xc / %noxc for XoverColoring(true / false) - default = true
    /// - %sr(r) for SphereRadius(r)
    /// - %sc(HHHHHHHH) for SphereColor(0xHHHHHHHH) or HHHHHH = a material color
    /// - %br(r) for BondRadius(r)
    /// - %bc(HHHHHHHH) for BondColor(0xHHHHHHHH)
    /// - %hr(r) for DoubleHelixAsCylinderRadius(r)
    /// - %hc(HHHHHHHH) for DoubleHelixAsCylinderColor(0xHHHHHHHH)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = s
            .split(&['%', ' ', ',', ')', '('])
            .filter(|x| x.len() > 0)
            .collect::<Vec<&str>>();

        match parsed.len() {
            1 => match parsed[0] {
                "rs" => return Ok(Self::RainbowStrand(true)),
                "nors" => return Ok(Self::RainbowStrand(false)),
                "noxc" => return Ok(Self::XoverColoring(false)),
                "xc" => return Ok(Self::XoverColoring(true)),
                _ => (),
            },
            2..=4 => {
                if let Ok(value) = f32::from_str(parsed[1]) {
                    match parsed[0] {
                        "sr" => return Ok(Self::SphereRadius(value / 10.)), // radius given in Å but stored in nm
                        "br" => return Ok(Self::BondRadius(value / 10.)), // radius given in Å but stored in nm
                        "hr" => return Ok(Self::DoubleHelixAsCylinderRadius(value / 10.)), // radius given in Å but stored in nm
                        _ => (),
                    }
                }
                let mut color = 0xFF_FF_FF_FF;
                let mut hue_range = None;
                if let Ok(value) = MaterialColor::from_str(parsed[1]) {
                    color = value as u32;
                } else if let Ok(value) = u32::from_str_radix(parsed[1], 16) {
                    color = value;
                }
                if parsed.len() > 2 {
                    if let Ok(alpha) = f32::from_str(parsed[2]) {
                        let alpha = (alpha * 255.).min(255.).max(0.).round() as u32;
                        color = (color & 0xFF_FF_FF) | (alpha << 24);
                        if parsed.len() > 3 {
                            if let Ok(h_range) = f64::from_str(parsed[3]) {
                                hue_range = Some(h_range);
                            }
                        }
                    }
                }

                match parsed[0] {
                    "sc" => return Ok(Self::SphereColor(color)),
                    "bc" => return Ok(Self::BondColor(color)),
                    "hc" => return Ok(Self::DoubleHelixAsCylinderColor(color)),
                    "cs" => return Ok(Self::ColorShade(color, hue_range)),
                    _ => (),
                }
            }
            _ => (),
        }

        return Err(ParsePointError);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct DrawingStyle {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sphere_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bond_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub helix_as_cylinder_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sphere_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bond_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub helix_as_cylinder_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rainbow_strand: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub xover_coloring: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color_shade: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hue_range: Option<f64>,
}

impl std::default::Default for DrawingStyle {
    fn default() -> Self {
        DrawingStyle {
            sphere_radius: None,
            bond_radius: None,
            helix_as_cylinder_radius: None,
            sphere_color: None,
            bond_color: None,
            helix_as_cylinder_color: None,
            rainbow_strand: None,
            xover_coloring: None,
            color_shade: None,
            hue_range: None,
        }
    }
}

impl From<Vec<DrawingAttribute>> for DrawingStyle {
    fn from(attributes: Vec<DrawingAttribute>) -> Self {
        let mut ret = DrawingStyle::default();
        for att in attributes {
            match att {
                DrawingAttribute::SphereRadius(r) => {
                    ret.sphere_radius = ret.sphere_radius.or(Some(r))
                }
                DrawingAttribute::BondRadius(r) => ret.bond_radius = ret.bond_radius.or(Some(r)),
                DrawingAttribute::DoubleHelixAsCylinderRadius(r) => {
                    ret.helix_as_cylinder_radius = ret.helix_as_cylinder_radius.or(Some(r))
                }
                DrawingAttribute::SphereColor(c) => ret.sphere_color = ret.sphere_color.or(Some(c)),
                DrawingAttribute::BondColor(c) => ret.bond_color = ret.bond_color.or(Some(c)),
                DrawingAttribute::DoubleHelixAsCylinderColor(c) => {
                    ret.helix_as_cylinder_color = ret.helix_as_cylinder_color.or(Some(c))
                }
                DrawingAttribute::RainbowStrand(b) => {
                    ret.rainbow_strand = ret.rainbow_strand.or(Some(b))
                }
                DrawingAttribute::XoverColoring(b) => {
                    ret.xover_coloring = ret.xover_coloring.or(Some(b))
                }
                DrawingAttribute::ColorShade(c, hue_range) => {
                    ret.color_shade = ret.color_shade.or(Some(c));
                    ret.hue_range = ret.hue_range.or(hue_range);
                }
            }
        }
        return ret;
    }
}

impl DrawingStyle {
    pub fn with_attribute(&self, att: DrawingAttribute) -> Self {
        match att {
            DrawingAttribute::SphereRadius(r) => DrawingStyle {
                sphere_radius: Some(r),
                ..*self
            },
            DrawingAttribute::BondRadius(r) => DrawingStyle {
                bond_radius: Some(r),
                ..*self
            },
            DrawingAttribute::DoubleHelixAsCylinderRadius(r) => DrawingStyle {
                helix_as_cylinder_radius: Some(r),
                ..*self
            },
            DrawingAttribute::SphereColor(c) => DrawingStyle {
                sphere_color: Some(c),
                ..*self
            },
            DrawingAttribute::BondColor(c) => DrawingStyle {
                bond_color: Some(c),
                ..*self
            },
            DrawingAttribute::DoubleHelixAsCylinderColor(c) => DrawingStyle {
                helix_as_cylinder_color: Some(c),
                ..*self
            },
            DrawingAttribute::RainbowStrand(b) => DrawingStyle {
                rainbow_strand: Some(b),
                ..*self
            },
            DrawingAttribute::XoverColoring(b) => DrawingStyle {
                xover_coloring: Some(b),
                ..*self
            },
            DrawingAttribute::ColorShade(c, hue_range) => DrawingStyle {
                color_shade: Some(c),
                hue_range,
                ..*self
            },
        }
    }

    pub fn attributes(&self) -> Vec<DrawingAttribute> {
        let mut atts = Vec::new();

        if let Some(r) = self.sphere_radius {
            atts.push(DrawingAttribute::SphereRadius(r))
        }
        if let Some(r) = self.bond_radius {
            atts.push(DrawingAttribute::BondRadius(r))
        }
        if let Some(r) = self.helix_as_cylinder_radius {
            atts.push(DrawingAttribute::DoubleHelixAsCylinderRadius(r))
        }

        if let Some(c) = self.sphere_color {
            atts.push(DrawingAttribute::SphereColor(c))
        }
        if let Some(c) = self.bond_color {
            atts.push(DrawingAttribute::BondColor(c))
        }
        if let Some(c) = self.helix_as_cylinder_color {
            atts.push(DrawingAttribute::DoubleHelixAsCylinderColor(c))
        }

        if let Some(b) = self.rainbow_strand {
            atts.push(DrawingAttribute::RainbowStrand(b))
        }

        if let Some(b) = self.xover_coloring {
            atts.push(DrawingAttribute::XoverColoring(b))
        }

        if let Some(c) = self.color_shade {
            atts.push(DrawingAttribute::ColorShade(c, self.hue_range))
        }

        return atts;
    }

    pub fn complete_with_attribute(&self, att: DrawingAttribute) -> Self {
        match att {
            DrawingAttribute::SphereRadius(r) => DrawingStyle {
                sphere_radius: self.sphere_radius.or(Some(r)),
                ..*self
            },
            DrawingAttribute::BondRadius(r) => DrawingStyle {
                bond_radius: self.bond_radius.or(Some(r)),
                ..*self
            },
            DrawingAttribute::DoubleHelixAsCylinderRadius(r) => DrawingStyle {
                helix_as_cylinder_radius: self.helix_as_cylinder_radius.or(Some(r)),
                ..*self
            },
            DrawingAttribute::SphereColor(c) => DrawingStyle {
                sphere_color: self.sphere_color.or(Some(c)),
                ..*self
            },
            DrawingAttribute::BondColor(c) => DrawingStyle {
                bond_color: self.bond_color.or(Some(c)),
                ..*self
            },
            DrawingAttribute::DoubleHelixAsCylinderColor(c) => DrawingStyle {
                helix_as_cylinder_color: self.helix_as_cylinder_color.or(Some(c)),
                ..*self
            },
            DrawingAttribute::RainbowStrand(b) => DrawingStyle {
                rainbow_strand: self.rainbow_strand.or(Some(b)),
                ..*self
            },
            DrawingAttribute::XoverColoring(b) => DrawingStyle {
                xover_coloring: self.xover_coloring.or(Some(b)),
                ..*self
            },
            DrawingAttribute::ColorShade(c, hue_range) => DrawingStyle {
                color_shade: self.color_shade.or(Some(c)),
                hue_range,
                ..*self
            },
        }
    }

    pub fn complete_with_attributes(&self, attributes: Vec<DrawingAttribute>) -> Self {
        let mut style = *self;
        for att in attributes {
            style = style.complete_with_attribute(att);
        }
        return style.clone();
    }

    pub fn complete_with(&self, other: &Self) -> Self {
        return DrawingStyle {
            sphere_radius: self.sphere_radius.or(other.sphere_radius),
            bond_radius: self.bond_radius.or(other.bond_radius),
            helix_as_cylinder_radius: self
                .helix_as_cylinder_radius
                .or(other.helix_as_cylinder_radius),
            sphere_color: self.sphere_color.or(other.sphere_color),
            bond_color: self.bond_color.or(other.bond_color),
            helix_as_cylinder_color: self
                .helix_as_cylinder_color
                .or(other.helix_as_cylinder_color),
            rainbow_strand: self.rainbow_strand.or(other.rainbow_strand),
            xover_coloring: self.xover_coloring.or(other.xover_coloring),
            color_shade: self.color_shade.or(other.color_shade),
            hue_range: self.hue_range.or(other.hue_range),
        };
    }
}
