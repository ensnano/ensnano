use crate::{
    Design,
    domains::{Domain, helix_interval::HelixInterval, sanitize_domains},
    helices::{Helices, Helix},
    parameters::HelixParameters,
    strands::{Strand, Strands, read_junctions},
    utils::serde::is_false,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, f64::consts::PI, fmt, sync::Arc};
use ultraviolet::{DVec3, Rotor3, Vec2, Vec3};

/// The main type of this crate, describing a DNA design.
#[derive(Serialize, Deserialize, Clone)]
pub struct CodenanoDesign {
    /// Version of this format.
    pub version: String,
    /// The vector of all helices used in this design. Helices have a
    /// position and an orientation in 3D.
    pub helices: Vec<CodenanoHelix>,
    /// The vector of strands.
    pub strands: Vec<CodenanoStrand>,
    /// Parameters of DNA geometry. This can be skipped (in JSON), or
    /// set to `None` in Rust, in which case a default set of
    /// parameters from the literature is used.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parameters: Option<CodenanoParameters>,
}

/// A DNA strand.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CodenanoStrand {
    /// The (ordered) vector of domains, where each domain is a
    /// directed interval of a helix.
    pub domains: Vec<CodenanoDomain>,
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
    pub color: Option<CodenanoColor>,
}

impl CodenanoStrand {
    /// Provide a default color to the strand.
    pub fn default_color(&self) -> CodenanoColor {
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
        CodenanoColor::Int(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
/// Colors.
pub enum CodenanoColor {
    /// Colors encoded as u32.
    Int(u32),
    /// Hexadecimal colors.
    Hex(String),
    /// Three distinct fields for red, green and blue.
    Rgb {
        /// Red field.
        r: u8,
        /// Green field.
        g: u8,
        /// Blue field.
        b: u8,
    },
}

impl CodenanoColor {
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
pub struct CodenanoDomain {
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
    /// In addition to the strand-level sequence, individual domains
    /// may have sequences too. The precedence has to be defined by
    /// the user of this library.
    pub sequence: Option<Cow<'static, str>>,
}

/// An iterator over all positions of a domain.
pub struct CodenanoDomainIter {
    start: isize,
    end: isize,
    forward: bool,
}

impl Iterator for CodenanoDomainIter {
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
pub struct CodenanoParameters {
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

impl CodenanoParameters {
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
pub struct CodenanoHelix {
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

impl fmt::Debug for CodenanoHelix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("").field(&self.position).finish()
    }
}

const KELLY: [CodenanoColor; 19] = [
    // 0xF2F3F4, // White
    // 0x222222, // Black,
    CodenanoColor::Int(0xF3C300),
    CodenanoColor::Int(0x875692), // 0xF38400, // Orange, too close to others
    CodenanoColor::Int(0xA1CAF1),
    CodenanoColor::Int(0xBE0032),
    CodenanoColor::Int(0xC2B280),
    CodenanoColor::Int(0x848482),
    CodenanoColor::Int(0x008856),
    CodenanoColor::Int(0xE68FAC),
    CodenanoColor::Int(0x0067A5),
    CodenanoColor::Int(0xF99379),
    CodenanoColor::Int(0x604E97),
    CodenanoColor::Int(0xF6A600),
    CodenanoColor::Int(0xB3446C),
    CodenanoColor::Int(0xDCD300),
    CodenanoColor::Int(0x882D17),
    CodenanoColor::Int(0x8DB600),
    CodenanoColor::Int(0x654522),
    CodenanoColor::Int(0xE25822),
    CodenanoColor::Int(0x2B3D26),
];

impl Domain {
    fn from_codenano(codenano_domain: &CodenanoDomain) -> Self {
        let interval = HelixInterval {
            helix: codenano_domain.helix as usize,
            start: codenano_domain.start,
            end: codenano_domain.end,
            forward: codenano_domain.forward,
            sequence: codenano_domain.sequence.clone(),
        };
        Self::HelixDomain(interval)
    }
}

impl Strand {
    fn from_codenano(codenano_strand: &CodenanoStrand) -> Self {
        let domains: Vec<Domain> = codenano_strand
            .domains
            .iter()
            .map(Domain::from_codenano)
            .collect();
        let sane_domains = sanitize_domains(&domains, codenano_strand.cyclic);
        let junctions = read_junctions(&sane_domains, codenano_strand.cyclic);
        Self {
            domains: sane_domains,
            sequence: codenano_strand.sequence.clone(),
            is_cyclic: codenano_strand.cyclic,
            junctions,
            color: codenano_strand
                .color
                .clone()
                .unwrap_or_else(|| codenano_strand.default_color())
                .as_int(),
            ..Default::default()
        }
    }
}

impl HelixParameters {
    fn from_codenano(codenano_param: &CodenanoParameters) -> Self {
        Self {
            rise: codenano_param.rise as f32,
            helix_radius: codenano_param.helix_radius as f32,
            bases_per_turn: codenano_param.bases_per_turn as f32,
            groove_angle: codenano_param.groove_angle as f32,
            inter_helix_gap: codenano_param.inter_helix_gap as f32,
            inclination: 0.0,
        }
    }
}

impl Helix {
    fn from_codenano(codenano_helix: &CodenanoHelix) -> Self {
        let position = Vec3::new(
            codenano_helix.position.x as f32,
            codenano_helix.position.y as f32,
            codenano_helix.position.z as f32,
        );

        let orientation = Rotor3::from_rotation_xz(-codenano_helix.yaw as f32)
            * Rotor3::from_rotation_xy(codenano_helix.pitch as f32)
            * Rotor3::from_rotation_yz(codenano_helix.roll as f32);

        Self {
            position,
            orientation,
            helix_parameters: None,
            grid_position: None,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: None,
            scale2d: None,
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }
}

impl Design {
    pub fn from_codenano(codenano_design: &CodenanoDesign) -> Self {
        let mut helices = BTreeMap::new();
        for (i, helix) in codenano_design.helices.iter().enumerate() {
            helices.insert(i, Arc::new(Helix::from_codenano(helix)));
        }

        let mut strands = BTreeMap::new();
        for (i, strand) in codenano_design.strands.iter().enumerate() {
            strands.insert(i, Strand::from_codenano(strand));
        }

        let helix_parameters = codenano_design
            .parameters
            .map(|p| HelixParameters::from_codenano(&p))
            .unwrap_or_default();

        Self {
            helices: Helices(Arc::new(helices)),
            strands: Strands(strands),
            helix_parameters: Some(helix_parameters),
            ..Default::default()
        }
    }
}
