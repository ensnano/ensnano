#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RapierParameters {
    pub simulation_type: RapierSimulationType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RapierSimulationType {
    Full,
    Rigid,
    Cut,
}

impl std::fmt::Display for RapierSimulationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RapierSimulationType::Full => "Full",
            RapierSimulationType::Rigid => "Rigid",
            RapierSimulationType::Cut => "Cut",
        })
    }
}

impl RapierParameters {
    const DEFAULT: RapierParameters = RapierParameters {
        simulation_type: RapierSimulationType::Cut,
        linear_damping: 0.06,
        angular_damping: 0.06,
        interbase_spring_stiffness: 10000.0,
        interbase_spring_damping: 1000.0,
        crossover_stiffness: 100.0,
        crossover_damping: 50.0,
        crossover_rest_length: 0.64,
        free_nucleotide_stiffness: 40000.0,
        free_nucleotide_damping: 4000.0,
        free_nucleotide_rest_length: 0.332,
        repulsion_strength: 0.1,
        repulsion_range: 5.0,
    };
}

impl Default for RapierParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
