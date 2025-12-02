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
}

impl RapierParameters {
    pub fn parameters_array(&self) -> [f32; 12] {
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
        ]
    }

    pub fn set_parameters_array(&mut self, array: &[f32]) {
        assert!(array.len() > 11);

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

impl RapierParameters {
    const DEFAULT: Self = Self {
        is_simulation_running: false,
        simulation_type: RapierSimulationType::KCut,
        k_cut_threshold: 8,
        linear_damping: 0.06,
        angular_damping: 0.6,
        interbase_spring_stiffness: 10.0,
        interbase_spring_damping: 1.0,
        crossover_stiffness: 100.0,
        crossover_damping: 50.0,
        crossover_rest_length: 1.28,
        free_nucleotide_stiffness: 40.0,
        free_nucleotide_damping: 40.0,
        free_nucleotide_rest_length: 0.7,
        repulsion_strength: 2.0,
        repulsion_range: 1.5,
    };
}

impl Default for RapierParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
