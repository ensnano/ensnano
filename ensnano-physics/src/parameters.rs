//! This module defines the simulation parameters. This is exported
//! so that the interface part of the program can directly construct
//! the relevant data types.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RapierParameters {
    pub is_simulation_running: bool,
    pub ignore_local_parameters: bool,
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
    pub entropic_spring_strength: f32,
    pub entropic_spring_damping: f32,
    pub squish_strength: f32,
    pub squish_damping: f32,
    pub squish_soft_cutoff: f32,
}

pub const RAPIER_FLOAT_PARAMETERS_COUNT: usize = 18;

impl RapierParameters {
    const DEFAULT: Self = Self {
        is_simulation_running: false,
        ignore_local_parameters: true,
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
        repulsion_strength: 0.15,
        repulsion_range: 6.0,
        brownian_motion_strength: 0.0,
        entropic_spring_strength: 3.0,
        entropic_spring_damping: 40.0,
        squish_strength: 0.0,
        squish_damping: 1.0,
        squish_soft_cutoff: 3.0,
    };

    pub fn parameters_array(&self) -> [f32; RAPIER_FLOAT_PARAMETERS_COUNT] {
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
            self.entropic_spring_strength,
            self.entropic_spring_damping,
            self.squish_strength,
            self.squish_damping,
            self.squish_soft_cutoff,
        ]
    }

    #[expect(clippy::missing_asserts_for_indexing)]
    pub fn set_parameters_array(&mut self, array: &[f32]) {
        assert!(array.len() >= RAPIER_FLOAT_PARAMETERS_COUNT);

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
        self.entropic_spring_strength = array[13];
        self.entropic_spring_damping = array[14];
        self.squish_strength = array[15];
        self.squish_damping = array[16];
        self.squish_soft_cutoff = array[17];
    }
}

impl Default for RapierParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
