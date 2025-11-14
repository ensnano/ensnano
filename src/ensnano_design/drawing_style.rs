use super::MaterialColor;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, PartialEq, Deserialize, Clone, Debug, Copy)]
pub enum ColorType {
    Plain(u32),
    // TODO: Rainbow,
}

impl ColorType {
    pub fn to_u32(self) -> u32 {
        match self {
            Self::Plain(color) => color,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DrawingAttribute {
    SphereRadius(f32),
    BondRadius(f32),
    SphereColor(ColorType),
    BondColor(ColorType),
    DoubleHelixAsCylinderRadius(f32),
    DoubleHelixAsCylinderColor(ColorType), // with alpha
    RainbowStrand(bool),
    XoverColoring(bool),
    ColorShade(u32, Option<f64>),
    WithCones(bool),
    OnAxis(bool),
    Curvature(f32, f32),
    Torsion(f32, f32),
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
    /// - %wc / %noc for WithCones(true / false) - default = true
    /// - %onaxis / %offaxis for OnAxis(true / false) - default = false
    /// - %cv(r_min, r_max) - show the curvature radius using Purple to Blue gradient the helix cylinder for radius within the range r_min..r_max
    /// - %to(t_min, t_max) - show the torsion using Blue to Purple gradient the helix cylinder for radius within the range to_min..to_max
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = s
            .split(&['%', ' ', ',', ')', '('])
            .filter(|x| !x.is_empty())
            .collect::<Vec<&str>>();

        let len = parsed.len();
        if len == 0 {
            return Err(ParsePointError);
        }

        match parsed[0] {
            "rs" => return Ok(Self::RainbowStrand(true)),
            "nors" => return Ok(Self::RainbowStrand(false)),
            "noxc" => return Ok(Self::XoverColoring(false)),
            "wc" => return Ok(Self::WithCones(true)),
            "noc" => return Ok(Self::WithCones(false)),
            "xc" => return Ok(Self::XoverColoring(true)),
            "onaxis" => return Ok(Self::OnAxis(true)),
            "offaxis" => return Ok(Self::OnAxis(false)),
            "sr" | "br" | "hr" if len == 2 => {
                if let Ok(value) = f32::from_str(parsed[1]) {
                    match parsed[0] {
                        "sr" => return Ok(Self::SphereRadius(value / 10.)), // radius given in Å but stored in nm
                        "br" => return Ok(Self::BondRadius(value / 10.)), // radius given in Å but stored in nm
                        "hr" => return Ok(Self::DoubleHelixAsCylinderRadius(value / 10.)), // radius given in Å but stored in nm
                        _ => (),
                    }
                }
            }
            "sc" | "bc" | "hc" | "cs" if (2..=4).contains(&len) => {
                let mut color = 0xFF_FF_FF_FF;
                let mut hue_range = None;
                if let Ok(value) = MaterialColor::from_str(parsed[1]) {
                    color = value as u32;
                } else if let Ok(value) = u32::from_str_radix(parsed[1], 16) {
                    color = value;
                }

                if len > 2
                    && let Ok(alpha) = f32::from_str(parsed[2])
                {
                    let alpha = (alpha * 255.).clamp(0., 255.).round() as u32;
                    color = (color & 0xFF_FF_FF) | (alpha << 24);
                    if parsed.len() > 3
                        && let Ok(h_range) = f64::from_str(parsed[3])
                    {
                        hue_range = Some(h_range);
                    }
                }

                match parsed[0] {
                    "sc" => return Ok(Self::SphereColor(ColorType::Plain(color))),
                    "bc" => return Ok(Self::BondColor(ColorType::Plain(color))),
                    "hc" => return Ok(Self::DoubleHelixAsCylinderColor(ColorType::Plain(color))),
                    "cs" => return Ok(Self::ColorShade(color, hue_range)),
                    _ => (),
                }
            }
            "cv" if len == 3 => {
                if let Ok(r_min) = f32::from_str(parsed[1])
                    && let Ok(r_max) = f32::from_str(parsed[2])
                {
                    return Ok(Self::Curvature(r_min, r_max));
                }
            }
            "to" if len == 3 => {
                if let Ok(t_min) = f32::from_str(parsed[1])
                    && let Ok(t_max) = f32::from_str(parsed[2])
                {
                    return Ok(Self::Torsion(t_min, t_max));
                }
            }
            _ => (),
        }

        Err(ParsePointError)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, Default)]
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
    pub helix_as_cylinder_color: Option<ColorType>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rainbow_strand: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub xover_coloring: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color_shade: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hue_range: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub with_cones: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub on_axis: Option<bool>,
    /// (r_min, r_max) display curvature on the helix cylinder with a gradient for radius from r_min to r_max
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub curvature: Option<(f32, f32)>,
    /// (t_min, t_max) display torsion on the helix cylinder with a gradient for torsion from t_min to t_max
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub torsion: Option<(f32, f32)>,
}

impl From<Vec<DrawingAttribute>> for DrawingStyle {
    fn from(attributes: Vec<DrawingAttribute>) -> Self {
        let mut ret = Self::default();
        for att in attributes {
            match att {
                DrawingAttribute::SphereRadius(r) => {
                    ret.sphere_radius = ret.sphere_radius.or(Some(r));
                }
                DrawingAttribute::BondRadius(r) => ret.bond_radius = ret.bond_radius.or(Some(r)),
                DrawingAttribute::DoubleHelixAsCylinderRadius(r) => {
                    ret.helix_as_cylinder_radius = ret.helix_as_cylinder_radius.or(Some(r));
                }
                DrawingAttribute::SphereColor(c) => {
                    ret.sphere_color = ret.sphere_color.or(Some(c.to_u32()));
                }
                DrawingAttribute::BondColor(c) => {
                    ret.bond_color = ret.bond_color.or(Some(c.to_u32()));
                }
                DrawingAttribute::DoubleHelixAsCylinderColor(c) => {
                    ret.helix_as_cylinder_color = ret.helix_as_cylinder_color.or(Some(c));
                }
                DrawingAttribute::RainbowStrand(b) => {
                    ret.rainbow_strand = ret.rainbow_strand.or(Some(b));
                }
                DrawingAttribute::XoverColoring(b) => {
                    ret.xover_coloring = ret.xover_coloring.or(Some(b));
                }
                DrawingAttribute::WithCones(b) => ret.with_cones = ret.with_cones.or(Some(b)),
                DrawingAttribute::ColorShade(c, hue_range) => {
                    ret.color_shade = ret.color_shade.or(Some(c));
                    ret.hue_range = ret.hue_range.or(hue_range);
                }
                DrawingAttribute::OnAxis(b) => ret.on_axis = ret.on_axis.or(Some(b)),
                DrawingAttribute::Curvature(r_min, r_max) => {
                    ret.curvature = ret.curvature.or(Some((r_min, r_max)));
                }
                DrawingAttribute::Torsion(t_min, t_max) => {
                    ret.torsion = ret.torsion.or(Some((t_min, t_max)));
                }
            }
        }
        ret
    }
}

impl DrawingStyle {
    pub fn complete_with_attribute(&self, att: DrawingAttribute) -> Self {
        match att {
            DrawingAttribute::SphereRadius(r) => Self {
                sphere_radius: self.sphere_radius.or(Some(r)),
                ..*self
            },
            DrawingAttribute::BondRadius(r) => Self {
                bond_radius: self.bond_radius.or(Some(r)),
                ..*self
            },
            DrawingAttribute::DoubleHelixAsCylinderRadius(r) => Self {
                helix_as_cylinder_radius: self.helix_as_cylinder_radius.or(Some(r)),
                ..*self
            },
            DrawingAttribute::SphereColor(c) => Self {
                sphere_color: self.sphere_color.or(Some(c.to_u32())),
                ..*self
            },
            DrawingAttribute::BondColor(c) => Self {
                bond_color: self.bond_color.or(Some(c.to_u32())),
                ..*self
            },
            DrawingAttribute::DoubleHelixAsCylinderColor(c) => Self {
                helix_as_cylinder_color: self.helix_as_cylinder_color.or(Some(c)),
                ..*self
            },
            DrawingAttribute::RainbowStrand(b) => Self {
                rainbow_strand: self.rainbow_strand.or(Some(b)),
                ..*self
            },
            DrawingAttribute::XoverColoring(b) => Self {
                xover_coloring: self.xover_coloring.or(Some(b)),
                ..*self
            },
            DrawingAttribute::WithCones(b) => Self {
                with_cones: self.with_cones.or(Some(b)),
                ..*self
            },
            DrawingAttribute::ColorShade(c, hue_range) => Self {
                color_shade: self.color_shade.or(Some(c)),
                hue_range,
                ..*self
            },
            DrawingAttribute::OnAxis(b) => Self {
                on_axis: self.on_axis.or(Some(b)),
                ..*self
            },
            DrawingAttribute::Curvature(r_min, r_max) => Self {
                curvature: self.curvature.or(Some((r_min, r_max))),
                ..*self
            },
            DrawingAttribute::Torsion(t_min, t_max) => Self {
                torsion: self.torsion.or(Some((t_min, t_max))),
                ..*self
            },
        }
    }

    pub fn complete_with_attributes(&self, attributes: Vec<DrawingAttribute>) -> Self {
        let mut style = *self;
        for att in attributes {
            style = style.complete_with_attribute(att);
        }
        style
    }

    pub fn complete_with(&self, other: &Self) -> Self {
        Self {
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
            with_cones: self.with_cones.or(other.with_cones),
            color_shade: self.color_shade.or(other.color_shade),
            hue_range: self.hue_range.or(other.hue_range),
            on_axis: self.on_axis.or(other.on_axis),
            curvature: self.curvature.or(other.curvature),
            torsion: self.torsion.or(other.torsion),
        }
    }
}
