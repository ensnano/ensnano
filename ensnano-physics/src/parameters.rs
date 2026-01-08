//! This module defines the simulation parameters. This is exported
//! so that the interface part of the program can directly construct
//! the relevant data types.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RapierParameters {
    pub is_simulation_running: bool,
    pub simulation_type: RapierSimulationType,
    pub k_cut_threshold: u32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub interbase_spring_stiffness: f32,
    pub interbase_spring_damping: f32,
    pub crossover_stiffness: f32,
    pub crossover_damping: f32,
    pub crossover_rest_length: f32,
    pub free_nucleotide_stiffness: f32,
    pub free_nucleotide_damping: f32,
    pub free_nucleotide_rest_length: f32,
    pub repulsion_strength: f32,
    pub repulsion_range: f32,
    pub brownian_motion_strength: f32,
}

impl RapierParameters {
    const DEFAULT: Self = Self {
        is_simulation_running: false,
        simulation_type: RapierSimulationType::Full,
        k_cut_threshold: 8,
        linear_damping: 0.06,
        angular_damping: 0.6,
        interbase_spring_stiffness: 100.0,
        interbase_spring_damping: 10.0,
        crossover_stiffness: 100.0,
        crossover_damping: 50.0,
        crossover_rest_length: 0.68,
        free_nucleotide_stiffness: 80.0,
        free_nucleotide_damping: 40.0,
        free_nucleotide_rest_length: 0.7,
        repulsion_strength: 0.2,
        repulsion_range: 1.3,
        brownian_motion_strength: 2.0,
    };

    pub fn parameters_array(&self) -> [f32; 13] {
        [
            self.linear_damping,
            self.angular_damping,
            self.interbase_spring_stiffness,
            self.interbase_spring_damping,
            self.crossover_stiffness,
            self.crossover_damping,
            self.crossover_rest_length,
            self.free_nucleotide_stiffness,
            self.free_nucleotide_damping,
            self.free_nucleotide_rest_length,
            self.repulsion_strength,
            self.repulsion_range,
            self.brownian_motion_strength,
        ]
    }

    pub fn set_parameters_array(&mut self, array: &[f32]) {
        assert!(array.len() > 12);

        self.linear_damping = array[0];
        self.angular_damping = array[1];
        self.interbase_spring_stiffness = array[2];
        self.interbase_spring_damping = array[3];
        self.crossover_stiffness = array[4];
        self.crossover_damping = array[5];
        self.crossover_rest_length = array[6];
        self.free_nucleotide_stiffness = array[7];
        self.free_nucleotide_damping = array[8];
        self.free_nucleotide_rest_length = array[9];
        self.repulsion_strength = array[10];
        self.repulsion_range = array[11];
        self.brownian_motion_strength = array[12];
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RapierSimulationType {
    Full,
    Rigid,
    Cut,
    KCut,
}

impl std::fmt::Display for RapierSimulationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Full => "Full",
            Self::Rigid => "Rigid",
            Self::Cut => "Cut",
            Self::KCut => "KCut",
        })
    }
}

impl Default for RapierParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
