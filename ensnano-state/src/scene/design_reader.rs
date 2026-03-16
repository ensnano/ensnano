use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StrandNucleotidesPositions {
    pub is_cyclic: bool,
    pub positions: Vec<[f32; 3]>,
    pub curvatures: Vec<f64>,
    pub torsions: Vec<f64>,
}

pub type Scalebar = (f32, f32, fn(f32, f32, f32) -> u32);
