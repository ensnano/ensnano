//! This module defines the simulation parameters. This is exported
//! so that the interface part of the program can directly construct
//! the relevant data types.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RapierParameters {
    pub is_simulation_running: bool,
    pub cap_ups: bool,
    pub target_ups: u32,
    pub dt: f32,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RapierFloatParameter {
    DeltaTime,
    RepulsionStrength,
    RepulsionRange,
    BrownianStrength,
    EntropicStrength,
    EntropicDamping,
    PlanarStrength,
    PlanarDamping,
    PlanarCutoff,
    LinearDamping,
    AngularDamping,
    InterbaseStiffness,
    InterbaseDamping,
    CrossoverStiffness,
    CrossoverDamping,
    CrossoverRestLength,
    FreeStiffness,
    FreeDamping,
    FreeRestLength,
}

pub const RAPIER_FLOAT_PARAMETERS_COUNT: usize = 19;

impl RapierFloatParameter {
    pub fn values() -> [Self; RAPIER_FLOAT_PARAMETERS_COUNT] {
        [
            Self::DeltaTime,
            Self::RepulsionStrength,
            Self::RepulsionRange,
            Self::BrownianStrength,
            Self::EntropicStrength,
            Self::EntropicDamping,
            Self::PlanarStrength,
            Self::PlanarDamping,
            Self::PlanarCutoff,
            Self::LinearDamping,
            Self::AngularDamping,
            Self::InterbaseStiffness,
            Self::InterbaseDamping,
            Self::CrossoverStiffness,
            Self::CrossoverDamping,
            Self::CrossoverRestLength,
            Self::FreeStiffness,
            Self::FreeDamping,
            Self::FreeRestLength,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::DeltaTime => "Delta time (dt)",
            Self::RepulsionStrength => "Repulsion strength",
            Self::RepulsionRange => "Repulsion range",
            Self::BrownianStrength => "Brownian motion strength",
            Self::EntropicStrength => "Entropic springs strength",
            Self::EntropicDamping => "Entropic springs damping",
            Self::PlanarStrength => "Planar squish strength",
            Self::PlanarDamping => "Planar squish damping",
            Self::PlanarCutoff => "Planar squish soft cutoff",
            Self::LinearDamping => "Linear damping",
            Self::AngularDamping => "Angular damping",
            Self::InterbaseStiffness => "Interbase spring stiffness",
            Self::InterbaseDamping => "Interbase spring damping",
            Self::CrossoverStiffness => "Crossover stiffness",
            Self::CrossoverDamping => "Crossover damping",
            Self::CrossoverRestLength => "Crossover rest length",
            Self::FreeStiffness => "Free nucleotide stiffness",
            Self::FreeDamping => "Free nucleotide damping",
            Self::FreeRestLength => "Free nucleotide rest length",
        }
    }

    pub fn live_editability(&self) -> bool {
        match self {
            Self::DeltaTime
            | Self::RepulsionStrength
            | Self::RepulsionRange
            | Self::BrownianStrength
            | Self::EntropicStrength
            | Self::EntropicDamping
            | Self::PlanarStrength
            | Self::PlanarDamping
            | Self::PlanarCutoff => true,
            Self::LinearDamping
            | Self::AngularDamping
            | Self::InterbaseStiffness
            | Self::InterbaseDamping
            | Self::CrossoverStiffness
            | Self::CrossoverDamping
            | Self::CrossoverRestLength
            | Self::FreeStiffness
            | Self::FreeDamping
            | Self::FreeRestLength => false,
        }
    }

    pub fn min_value(&self) -> f32 {
        match self {
            Self::DeltaTime | Self::RepulsionRange => 0.0001,
            Self::RepulsionStrength
            | Self::BrownianStrength
            | Self::EntropicStrength
            | Self::PlanarStrength
            | Self::CrossoverStiffness
            | Self::InterbaseStiffness
            | Self::FreeStiffness
            | Self::EntropicDamping
            | Self::PlanarDamping
            | Self::LinearDamping
            | Self::AngularDamping
            | Self::InterbaseDamping
            | Self::CrossoverDamping
            | Self::FreeDamping
            | Self::CrossoverRestLength
            | Self::FreeRestLength
            | Self::PlanarCutoff => 0.0,
        }
    }

    pub fn max_value(&self) -> f32 {
        match self {
            Self::DeltaTime => 0.5,
            Self::RepulsionRange
            | Self::CrossoverRestLength
            | Self::FreeRestLength
            | Self::BrownianStrength => 10.0,
            Self::FreeStiffness | Self::CrossoverStiffness | Self::InterbaseStiffness => 500.0,
            Self::RepulsionStrength | Self::EntropicStrength | Self::PlanarStrength => 100.0,
            Self::EntropicDamping
            | Self::PlanarDamping
            | Self::LinearDamping
            | Self::AngularDamping
            | Self::InterbaseDamping
            | Self::CrossoverDamping
            | Self::FreeDamping => 250.0,
            Self::PlanarCutoff => 20.0,
        }
    }

    pub fn increment(&self) -> f32 {
        match self {
            Self::DeltaTime => 1.0 / 120.0,
            Self::RepulsionRange | Self::CrossoverRestLength | Self::FreeRestLength => 0.1,
            Self::FreeStiffness
            | Self::CrossoverStiffness
            | Self::InterbaseStiffness
            | Self::RepulsionStrength
            | Self::BrownianStrength
            | Self::EntropicStrength
            | Self::PlanarStrength
            | Self::EntropicDamping
            | Self::PlanarDamping
            | Self::LinearDamping
            | Self::AngularDamping
            | Self::InterbaseDamping
            | Self::CrossoverDamping
            | Self::FreeDamping => 1.0,
            Self::PlanarCutoff => 0.5,
        }
    }
}

impl RapierParameters {
    const DEFAULT: Self = Self {
        is_simulation_running: false,
        cap_ups: false,
        target_ups: 24,
        dt: 1.0 / 60.0,
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

    fn parameters_array(&self) -> [f32; RAPIER_FLOAT_PARAMETERS_COUNT] {
        [
            self.dt,
            self.repulsion_strength,
            self.repulsion_range,
            self.brownian_motion_strength,
            self.entropic_spring_strength,
            self.entropic_spring_damping,
            self.squish_strength,
            self.squish_damping,
            self.squish_soft_cutoff,
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
        ]
    }

    fn parameters_array_mut(&mut self) -> [&mut f32; RAPIER_FLOAT_PARAMETERS_COUNT] {
        [
            &mut self.dt,
            &mut self.repulsion_strength,
            &mut self.repulsion_range,
            &mut self.brownian_motion_strength,
            &mut self.entropic_spring_strength,
            &mut self.entropic_spring_damping,
            &mut self.squish_strength,
            &mut self.squish_damping,
            &mut self.squish_soft_cutoff,
            &mut self.linear_damping,
            &mut self.angular_damping,
            &mut self.interbase_spring_stiffness,
            &mut self.interbase_spring_damping,
            &mut self.crossover_stiffness,
            &mut self.crossover_damping,
            &mut self.crossover_rest_length,
            &mut self.free_nucleotide_stiffness,
            &mut self.free_nucleotide_damping,
            &mut self.free_nucleotide_rest_length,
        ]
    }

    pub fn set_parameter(&mut self, parameter: RapierFloatParameter, value: f32) {
        *self.parameters_array_mut()[parameter as usize] = value;
    }

    #[must_use]
    pub fn with_parameter(mut self, parameter: RapierFloatParameter, value: f32) -> Self {
        self.set_parameter(parameter, value);
        self
    }

    pub fn get_parameter(&self, parameter: RapierFloatParameter) -> f32 {
        self.parameters_array()[parameter as usize]
    }
}

impl Default for RapierParameters {
    fn default() -> Self {
        Self::DEFAULT
    }
}
