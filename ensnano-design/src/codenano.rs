use crate::utils::is_false;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, f64::consts::PI, fmt};
use ultraviolet::DVec3;

/// The main type of this crate, describing a DNA design.
#[derive(Serialize, Deserialize, Clone)]
pub struct Design<StrandLabel, DomainLabel> {
    /// Version of this format.
    pub version: String,
    /// The vector of all helices used in this design. Helices have a
    /// position and an orientation in 3D.
    pub helices: Vec<Helix>,
    /// The vector of strands.
    pub strands: Vec<Strand<StrandLabel, DomainLabel>>,
    /// Parameters of DNA geometry. This can be skipped (in JSON), or
    /// set to `None` in Rust, in which case a default set of
    /// parameters from the literature is used.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parameters: Option<Parameters>,
}

/// A DNA strand.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Strand<StrandLabel, DomainLabel> {
    /// The (ordered) vector of domains, where each domain is a
    /// directed interval of a helix.
    pub domains: Vec<Domain<DomainLabel>>,
    /// The sequence of this strand, if any. If the sequence is longer
    /// than specified by the domains, a prefix is assumed. Can be
    /// skipped in the serialization.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sequence: Option<Cow<'static, str>>,
    /// Is this sequence cyclic? Can be skipped (and defaults to
    /// `false`) in the serialization.
    #[serde(skip_serializing_if = "is_false", default)]
    pub cyclic: bool,
    /// Color of this strand. If skipped, a default color will be
    /// chosen automatically.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub color: Option<Color>,
    /// An optional label for the strand. Can be
    /// `serde_json::Value::Null`, and skipped in the serialization.
    #[serde(skip_serializing_if = "Option::is_none", default = "none")]
    pub label: Option<StrandLabel>,
}

fn none<T>() -> Option<T> {
    None
}

impl<StrandLabel, DomainLabel> Strand<StrandLabel, DomainLabel> {
    /// Provide a default color to the strand.
    pub fn default_color(&self) -> Color {
        if let Some(domain) = self.domains.first() {
            let x1 = if domain.forward {
                domain.end - 1
            } else {
                domain.start
            };
            let h = domain.helix;
            let x = x1 + (x1 % 11) + 5 * h;
            let n = KELLY.len() as isize;
            return KELLY[(((x % n) + n) % n) as usize].clone();
        }
        Color::Int(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Colors
pub enum Color {
    /// Colors encoded as u32.
    Int(u32),
    /// Hexadecimal colors
    Hex(String),
    /// Three distinct fields for red, green and blue
    Rgb {
        /// Red field
        r: u8,
        /// Green field
        g: u8,
        /// Blue field
        b: u8,
    },
}

impl Color {
    /// Returns the u32 encoding this color.
    pub fn as_int(&self) -> u32 {
        match self {
            Self::Int(n) => *n,
            Self::Hex(s) => {
                let s = s.trim_start_matches("0x");
                let s = s.trim_start_matches('#');
                u32::from_str_radix(s, 16).unwrap()
            }
            Self::Rgb { r, g, b } => ((*r as u32) << 16) | ((*g as u32) << 8) | (*b as u32),
        }
    }
}

/// A domain, i.e. an interval of a helix.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Domain<StrandLabel> {
    /// Index of the helix in the array of helices. Indices start at
    /// 0.
    pub helix: isize,
    /// Position of the leftmost base of this domain along the helix
    /// (this might be the first or last base of the domain, depending
    /// on the `orientation` parameter below).
    pub start: isize,
    /// Position of the first base after the forwardmost base of the
    /// domain, along the helix. Domains must always be such that
    /// `domain.start < domain.end`.
    pub end: isize,
    /// If true, the "5' to 3'" direction of this domain runs in the
    /// same direction as the helix, i.e. "to the forward" along the
    /// axis of the helix. Else, the 5' to 3' runs to the left along
    /// the axis.
    pub forward: bool,
    /// An optional label that can be attached to strands.
    #[serde(skip_serializing_if = "Option::is_none", default = "none")]
    pub label: Option<StrandLabel>,
    /// In addition to the strand-level sequence, individual domains
    /// may have sequences too. The precedence has to be defined by
    /// the user of this library.
    pub sequence: Option<Cow<'static, str>>,
}

/// An iterator over all positions of a domain.
pub struct DomainIter {
    start: isize,
    end: isize,
    forward: bool,
}

impl Iterator for DomainIter {
    type Item = isize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else if self.forward {
            let s = self.start;
            self.start += 1;
            Some(s)
        } else {
            let s = self.end;
            self.end -= 1;
            Some(s - 1)
        }
    }
}

/// DNA geometric parameters.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Parameters {
    /// Distance between two consecutive bases along the axis of a
    /// helix, in nanometers.
    #[serde(alias = "z_step")]
    pub rise: f64,
    /// Radius of a helix, in nanometers.
    pub helix_radius: f64,
    /// Number of bases per turn in nanometers.
    pub bases_per_turn: f64,
    /// Minor groove angle. DNA helices have a "minor groove" and a
    /// "major groove", meaning that two paired nucleotides are not at
    /// opposite positions around a double helix (i.e. at an angle of
    /// 180°), but instead have a different angle.
    ///
    /// Strands are directed. The "normal" direction is called "5' to
    /// 3'" (named after parts of the nucleotides). This parameter is
    /// the small angle, which is clockwise from the normal strand to
    /// the reverse strand.
    pub groove_angle: f64,

    /// Gap between two neighboring helices.
    pub inter_helix_gap: f64,
}

impl Parameters {
    /// Default values for the parameters of DNA, taken from the literature.
    pub const DEFAULT: Self = Self {
        // z-step and helix radius from:
        //
        // Single-molecule portrait of DNA and RNA double helices,
        // J. Ricardo Arias-Gonzalez, Integrative Biology, Royal
        // Society of Chemistry, 2014, vol. 6, p.904
        rise: 0.332,
        helix_radius: 1.,
        // bases per turn from Woo Rothemund (Nature Chemistry).
        bases_per_turn: 10.44,
        groove_angle: -24. * PI / 34.,
        // From Paul's paper.
        inter_helix_gap: 0.65,
    };
}

/// A DNA helix. All bases of all strands must be on a helix.
///
/// The three angles are illustrated in the following image, from [the NASA website](https://www.grc.nasa.gov/www/k-12/airplane/rotations.html):
///
/// ![Aircraft angles](https://www.grc.nasa.gov/www/k-12/airplane/Images/rotations.gif)
#[derive(Serialize, Deserialize, Clone)]
pub struct Helix {
    /// Position of the position of the helix axis.
    #[serde(default)]
    pub position: DVec3,

    /// Angle around the axis of the helix.
    #[serde(default)]
    pub roll: f64,

    /// Horizontal rotation.
    #[serde(default)]
    pub yaw: f64,

    /// Vertical rotation.
    #[serde(default)]
    pub pitch: f64,

    /// Maximum available position of the helix.
    pub max_offset: Option<isize>,

    /// Bold tickmarks.
    pub major_ticks: Option<Vec<isize>>,
}

impl fmt::Debug for Helix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("").field(&self.position).finish()
    }
}

const KELLY: [Color; 19] = [
    // 0xF2F3F4, // White
    // 0x222222, // Black,
    Color::Int(0xF3C300),
    Color::Int(0x875692), // 0xF38400, // Orange, too close to others
    Color::Int(0xA1CAF1),
    Color::Int(0xBE0032),
    Color::Int(0xC2B280),
    Color::Int(0x848482),
    Color::Int(0x008856),
    Color::Int(0xE68FAC),
    Color::Int(0x0067A5),
    Color::Int(0xF99379),
    Color::Int(0x604E97),
    Color::Int(0xF6A600),
    Color::Int(0xB3446C),
    Color::Int(0xDCD300),
    Color::Int(0x882D17),
    Color::Int(0x8DB600),
    Color::Int(0x654522),
    Color::Int(0xE25822),
    Color::Int(0x2B3D26),
];
